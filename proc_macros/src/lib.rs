use proc_macro::{Span, TokenStream};
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Expr, ExprField, Field, FieldValue, FieldsNamed, FieldsUnnamed, Ident, Index, ItemFn, Token,
    Type,
    parse::Parse,
    parse_macro_input,
    punctuated::{Pair, Punctuated},
    spanned::Spanned,
    token::{Colon, Comma, Dot},
};

#[proc_macro_attribute]
pub fn bevy_app(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(item as ItemFn);
    let name = &input_fn.sig.ident;
    let expanded = quote! {
        struct BevyExtensionLibrary;

        #[gdextension]
        unsafe impl ExtensionLibrary for BevyExtensionLibrary {
            fn on_level_init(level: godot::prelude::InitLevel) {
                if level == godot::prelude::InitLevel::Core {
                    godot::private::class_macros::registry::class::auto_register_classes(level);
                    let mut app_builder_func = bevy_godot4::APP_BUILDER_FN.lock().unwrap();
                    if app_builder_func.is_none() {
                        *app_builder_func = Some(Box::new(#name));
                    }
                }
            }
        }
        #input_fn
    };

    expanded.into()
}

struct Args {
    name: Ident,
    comma: Option<Token!(,)>,
    fields: Punctuated<Field, Token!(,)>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let comma = input.parse::<Comma>().ok(); // this may have unintended consequences :P
        let fields: Punctuated<Field, Token!(,)> = input.parse_terminated(Field::parse_named)?;

        Ok(Self {
            name,
            comma,
            fields,
        })
    }
}

struct ArgsInstance {
    name: Ident,
    comma: Token!(,),
    instance: Type,
    comma2: Option<Token!(,)>,
    fields: Punctuated<Field, Token!(,)>,
}

impl Parse for ArgsInstance {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let comma = input.parse::<Comma>()?;
        let instance = input.parse::<Type>()?;
        let comma2 = input.parse::<Comma>().ok(); // this may have unintended consequences :P
        let fields: Punctuated<Field, Token!(,)> = input.parse_terminated(Field::parse_named)?;

        Ok(Self {
            name,
            comma,
            instance,
            comma2,
            fields,
        })
    }
}

#[proc_macro]
pub fn signal_event(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as Args);
    let name = input.name;
    let fields = input.fields;
    let types_raw = fields
        .iter()
        .map::<Pair<Field, Token!(,)>, _>(|field| Pair::Punctuated(field.clone(), Comma::default()))
        .collect::<Punctuated<Field, Token!(,)>>();
    let types_to_params = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = field.clone().ident.unwrap();
            quote!(#name: params.#i)
        })
        // .map::<Pair<FieldValue, Token!(,)>, _>(|(i, field)| {
        //     Pair::Punctuated(
        //         FieldValue {
        //             attrs: Vec::new(),
        //             member: field.ident.clone().unwrap().into(),
        //             colon_token: Some(Colon::default()),
        //             expr: syn::Expr::Field(ExprField {
        //                 attrs: Vec::new(),
        //                 base: Box::new(Expr::Verbatim(
        //                     Ident::new("params", field.span()).to_token_stream(),
        //                 )),
        //                 dot_token: Dot::default(),
        //                 member: syn::Member::Unnamed(Index::from(i)),
        //             }),
        //         },
        //         Comma::default(),
        //     )
        // })
        // .collect::<Punctuated<FieldValue, Token!(,)>>();
        .collect::<TokenStream2>();

    let types = FieldsUnnamed {
        paren_token: syn::token::Paren {
            span: types_raw.span(),
        },
        unnamed: types_raw,
    };

    let x = quote! {
        #[derive(bevy::prelude::Event)]
        pub struct #name {
            #fields
        }

        impl std::convert::From<#types> for #name {
            fn from(params: #types) -> Self {
                Self {
                    #types_to_params
                }
            }
        }
    };

    x.to_token_stream().into()
}

#[proc_macro]
pub fn signal_event_instanced(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ArgsInstance);
    let name = input.name;
    let instance = input.instance;
    let fields = input.fields;
    let types_raw = fields
        .iter()
        .map::<Pair<Field, Token!(,)>, _>(|field| Pair::Punctuated(field.clone(), Comma::default()))
        .collect::<Punctuated<Field, Token!(,)>>();
    let types_to_params = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = field.clone().ident.unwrap();
            quote!(#name: params.1.#i)
        })
        .collect::<TokenStream2>();

    let types = FieldsUnnamed {
        paren_token: syn::token::Paren {
            span: types_raw.span(),
        },
        unnamed: types_raw,
    };

    let x = quote! {
        #[derive(bevy::prelude::Event)]
        pub struct #name {
            instance: bevy_godot4::prelude::TypedErasedGd<#instance>,
            #fields
        }

        impl std::convert::From<(Gd<#instance>, #types)> for #name {
            fn from(params: (Gd<#instance>, #types)) -> Self {
                Self {
                    instance: bevy_godot4::prelude::TypedErasedGd::new(params.0)
                    #types_to_params
                }
            }
        }
    };

    x.to_token_stream().into()
}
