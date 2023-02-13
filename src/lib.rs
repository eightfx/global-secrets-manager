//! 	# What is this?
//!
//! This is a procedural macro for easy use of AWS Secrets Manager. 
//! 	This code allows you to create a global constant of the same type as the name of Secrets Manager by simply creating a structure that matches the key pair set in Secrets Manager. 
//! 	This way, you can access the secret values stored in Secrets Manager without writing any code to fetch them from AWS. 
//!
//! Pros:
//! - Key pairs can be retrieved by simply defining a struct with the same structure as the key pair set in Secrets Manager
//! - Key pairs are defined as global constants, so they can be used from anywhere
//! - Lazy evaluation by once_cell::sync::Lazy
//!
//! # Example code
//!
//!     use global_secrets_manager::GlobalSecretsManager;
//!
//!     /// Please use the same name as Secrets Manager for the name of the structure
//!     /// Please set the keys of Secrets Manager without any omission or excess
//!     #[derive(GlobalSecretsManager)]
//!     #[derive(Debug, serde::Deserialize)]
//!     pub struct SampleSecrets{ 
//!         key1: String,
//!         key2: String,
//!     }
//!
//!
//!
//!     fn main(){
//!         dbg!(&SampleSecrets.key1); //-> value1
//!         dbg!(&SampleSecrets.key2); //-> value2
//!     }
//!
//! # Advance Preparation
//!
//!
//! ## Dependencies
//!
//! The following dependencies are required.
//!
//!     aws-config = "0.54.1"
//!     aws-sdk-secretsmanager = "0.24.0"
//!     once_cell = "1.17.0"
//!     dotenvy = "0.15.6"
//!     serde_json = "1.0.93"
//!     tokio = { version = "1.21.2", features = ["full"] }
//!     global-secrets-manager = "0.1.1"
//!
//! However, it is better to use the latest versions of them.
//!
//!
//! ## AWS Secrets Manager settings
//!
//! Please set up your secrets in AWS Secrets Manager according to the relevant page.
//! For the sake of explanation, let's assume that the name of Secrets Manager is SampleSecrets and the secret values are set as follows.
//!
//! <table border="2" cellspacing="0" cellpadding="6" rules="groups" frame="hsides">
//!
//! <colgroup>
//! <col  class="org-left" />
//!
//! <col  class="org-left" />
//! </colgroup>
//! <thead>
//! <tr>
//! <th scope="col" class="org-left">Secret Key</th>
//! <th scope="col" class="org-left">Secret Value</th>
//! </tr>
//! </thead>
//!
//! <tbody>
//! <tr>
//! <td class="org-left">key1</td>
//! <td class="org-left">value1</td>
//! </tr>
//!
//!
//! <tr>
//! <td class="org-left">key2</td>
//! <td class="org-left">value2</td>
//! </tr>
//! </tbody>
//! </table>
//!
//!
//! ## AWS credential acquisition
//!
//! Please obtain your credential information.
//! If you are using AWS CLI, you can get it with the following command.
//!
//!     cat ~/.aws/credentials
//!
//!
//! ## .env settings
//!
//! Create a .env file in your repository and enter your credential information as follows.
//!
//!     AWS_ACCESS_KEY_ID=AAAAA
//!     AWS_SECRET_ACCESS_KEY = BBBBB
//!     AWS_REGION = CCCCC
//!
//!
//!
//! ## Explanation of internal specifications
//!
//! For the structure
//!
//!     struct SampleSecrets{
//!         key1:String,
//!         key2:String
//!     }
//!
//! the same name global constant
//!
//!     pub static SampleSecrets: once_cell::sync::Lazy<SampleSecrets> = once_cell::sync::Lazy::new(||SampleSecrets::get());
//!
//! is defined. This constant is initialized only once when it is first accessed, and it calls the get() method of the structure to fetch the secret values from AWS Secrets Manager.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};


#[proc_macro_derive(GlobalSecretsManager)]
pub fn GlobalSecretsManager_derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input as DeriveInput);
    let fields = match data {
        syn::Data::Struct(data) => data.fields,
        _ => panic!("Global_sm can only be applied to structs"),
    };

    let field_names: Vec<_> = fields
        .iter()
        .map(|f| f.ident.as_ref().expect("expected named field"))
        .collect();
    let field_types: Vec<_> = fields.iter().map(|f| &f.ty).collect();

    let expanded = quote! {
        pub static #ident: once_cell::sync::Lazy<#ident> = once_cell::sync::Lazy::new(||#ident::get());

        impl #ident {
            async fn get_secret() -> String {
                let shared_config = aws_config::from_env().load().await;
                let client = aws_sdk_secretsmanager::Client::new(&shared_config);
                let resp = client.get_secret_value().secret_id(std::any::type_name::<Self>().split("::").last().unwrap()).send().await.unwrap();
                let secret = resp.secret_string.unwrap();
                secret
            }

            fn get() -> Self {
                dotenvy::dotenv().ok();
                let rt = tokio::runtime::Runtime::new().unwrap();
                let rt_str: String = rt.block_on(Self::get_secret());
                let rt_bytes: &'static [u8] = Box::leak(rt_str.into_bytes().into_boxed_slice()); 
                let secret_keys: Self = serde_json::from_slice(rt_bytes).unwrap(); 
                secret_keys
            }
        }
    };

    TokenStream::from(expanded)
}
