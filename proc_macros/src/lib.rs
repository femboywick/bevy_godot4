use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{ToTokens, quote};
use syn::{
    Field, FieldsNamed, FieldsUnnamed, Ident, ItemFn, ItemType, Token, Type,
    ext::IdentExt,
    parse::{Parse, ParseBuffer, Parser},
    parse_macro_input, parse_quote, parse_str,
    punctuated::{Pair, Punctuated},
    spanned::Spanned,
    token::Comma,
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
    _comma: Option<Token!(,)>,
    fields: Punctuated<Field, Token!(,)>,
}

impl Parse for Args {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let _comma = input.parse::<Comma>().ok(); // this may have unintended consequences :P
        let fields: Punctuated<Field, Token!(,)> = input.parse_terminated(Field::parse_named)?;

        Ok(Self {
            name,
            _comma,
            fields,
        })
    }
}

struct ArgsInstance {
    name: Ident,
    _comma: Token!(,),
    instance: Type,
    _comma2: Option<Token!(,)>,
    fields: Punctuated<Field, Token!(,)>,
}

impl Parse for ArgsInstance {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?;
        let _comma = input.parse::<Comma>()?;
        let instance = input.parse::<Type>()?;
        let _comma2 = input.parse::<Comma>().ok(); // this may have unintended consequences :P
        let fields: Punctuated<Field, Token!(,)> = input.parse_terminated(Field::parse_named)?;

        Ok(Self {
            name,
            _comma,
            instance,
            _comma2,
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
        .map::<TokenStream2, _>(|field| {
            let ident = field.ty.clone();
            quote!(#ident,)
        })
        .collect::<TokenStream2>();
    let types = quote!((#types_raw));
    let types_to_params = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = field.clone().ident;
            quote!(#name: params.1.#i)
        })
        .collect::<TokenStream2>();

    let x = quote! {
        #[derive(bevy::prelude::Event)]
        pub struct #name {
            #fields
        }

        impl std::convert::From<((), #types)> for #name {
            fn from(params: ((), #types)) -> Self {
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
    let fields: Vec<Field> = input
        .fields
        .into_iter()
        .map(|field| {
            let mut token_tree = field.ty.to_token_stream().into_iter();
            let Some(token_tree1) = token_tree.nth(0) else {
                return field;
            };
            let Some(ty) = token_tree.nth(1) else {
                return field;
            };
            let name = field.ident.clone();
            if token_tree1.to_string() == "Gd" {
                let ident = Ident::parse_any
                    .parse2(quote!(bevy_godot4::prelude::TypedErasedGd<#ty>))
                    .unwrap();
                return Field::parse_named.parse2(quote!(#name: #ident)).unwrap();
            } else if token_tree1.to_string() == "DynGd" {
                let ident = Ident::parse_any
                    .parse2(quote!(bevy_godot4::prelude::DynErasedGd<#ty>))
                    .unwrap();
                return Field::parse_named.parse2(quote!(#name: #ident)).unwrap();
            } else {
                return field;
            }
        })
        .collect();
    let types_raw = fields
        .iter()
        .map::<TokenStream2, _>(|field| {
            let ident = field.ty.clone();
            quote!(#ident,)
        })
        .collect::<TokenStream2>();
    let types = quote!((#types_raw));
    let types_to_params = fields
        .iter()
        .enumerate()
        .map(|(i, field)| {
            let name = field.clone().ident;
            if field
                .clone()
                .ty
                .into_token_stream()
                .to_string()
                .starts_with("Gd")
            {
                return quote!(#name: bevy_godot4::prelude::TypedErasedGd::new(params.1.#i));
            } else if field
                .clone()
                .ty
                .into_token_stream()
                .to_string()
                .starts_with("DynGd")
            {
                return quote!(#name: bevy_godot4::prelude::DynErasedGd::new(params.1.#i));
            }
            quote!(#name: params.1.#i)
        })
        .collect::<TokenStream2>();

    let fields_token = fields
        .iter()
        .map(|field| quote!(#field,))
        .collect::<TokenStream2>();

    let x = quote! {
        #[derive(bevy::prelude::Event)]
        pub struct #name {
            instance: bevy_godot4::prelude::TypedErasedGd<#instance>,
            #fields_token
        }

        impl std::convert::From<(Gd<#instance>, #types)> for #name {
            fn from(params: (Gd<#instance>, #types)) -> Self {
                Self {
                    instance: bevy_godot4::prelude::TypedErasedGd::new(params.0),
                    #types_to_params
                }
            }
        }
    };

    x.to_token_stream().into()
}
