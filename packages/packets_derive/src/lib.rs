use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, Token, parse_macro_input, punctuated::Punctuated};

#[proc_macro_derive(Packets)]
pub fn packets(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    let data = match input.data {
        syn::Data::Enum(data) => data,
        _ => panic!("Unsupported data type"),
    };

    let mut from_string_map: Punctuated<_, Token![,]> = Punctuated::new();
    let mut to_string_map: Punctuated<_, Token![,]> = Punctuated::new();

    for ele in &data.variants {
        let ident = &ele.ident;
        let package_name = format!(
            "{}{}",
            &ident.to_string()[..1].to_lowercase(),
            &ident.to_string()[1..]
        );
        let package_name_with_brackets = format!("[{}]", package_name);

        if ele.fields.is_empty() {
            from_string_map.push(quote! {
              #package_name_with_brackets => {Ok(#name::#ident)}
            });

            to_string_map.push(quote! {
              #name::#ident => (#package_name, None)
            });
        } else {
            from_string_map.push(quote! {
              #package_name_with_brackets => {Ok(#name::#ident(get_content(content)?)) }
            });

            to_string_map.push(quote! {
              #name::#ident(content) => (#package_name, Some(serde_json::to_string(content).unwrap()))
            });
        }
    }

    let expended = quote! {
       impl #name {
        pub fn from_string(message: &str) -> Result<Self, PacketError> {
          let mut split = message.splitn(2, " ");

          let name = split.next().unwrap().to_string();
          let content = split.next().map(|content| content.to_string());

          fn get_content<T: DeserializeOwned>(content: Option<String>) -> Result<T, PacketError> {
              serde_json::from_str(&content.ok_or(PacketError::InvalidPacket)?)
                  .map_err(|_| PacketError::InvalidPacket)
          }

          match name.as_str() {
            #from_string_map
            _ => Err(PacketError::UnknownPacket)
          }
        }
      }

      impl Display for #name {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
          let (name, content) = match self {
            #to_string_map
          };

          if let Some(content) = content {
              f.write_fmt(format_args!("[{}] {}", name, content))
          } else {
              f.write_fmt(format_args!("[{}]", name))
          }
        }
      }
    };

    TokenStream::from(expended)
}
