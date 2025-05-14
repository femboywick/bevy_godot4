use proc_macro::TokenStream;
use quote::quote;
use syn::{ItemFn, parse_macro_input};

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
                else if level == godot::prelude::InitLevel::Scene {
                    // The `&str` identifies your singleton and can be
                    // used later to access it.
                    godot::classes::Engine::singleton().register_singleton(
                        "BevyAppSingleton",
                        &BevyApp::new_alloc(),
                    );
                }

            fn on_level_deinit(level: godot::prelude::InitLevel) {
                if level == godot::prelude::InitLevel::Scene {
                    // Let's keep a variable of our Engine singleton instance,
                    // and MyEngineSingleton name.
                    let mut engine = godot::classes::Engine::singleton();
                    let singleton_name = "MyEngineSingleton";

                    // Here, we manually retrieve our singleton(s) that we've registered,
                    // so we can unregister them and free them from memory - unregistering
                    // singletons isn't handled automatically by the library.
                    if let Some(my_singleton) = engine.get_singleton(singleton_name) {
                        // Unregistering from Godot, and freeing from memory is required
                        // to avoid memory leaks, warnings, and hot reloading problems.
                        engine.unregister_singleton(singleton_name);
                        my_singleton.free();
                    } else {
                        // You can either recover, or panic from here.
                        godot_error!("Failed to get singleton");
                    }
                }
            }

        }


                }

    };

    expanded.into()
}
