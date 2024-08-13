use attribute_derive::FromAttr;
use manyhow::ensure;
use syn::{Data, DataStruct, DeriveInput, Fields, Variant};

use super::*;

pub fn append(
    DeriveInput {
        attrs,
        ident,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result {
    let value_signature = Option::<ValueSignature>::from_attributes(attrs)?;
    let body = match data {
        Data::Struct(DataStruct {
            fields: Fields::Named(fields),
            ..
        }) => named_struct(fields.named, value_signature),
        Data::Struct(_) => todo!(),
        Data::Enum(data) => {
            if let Some(discriminant) = data.variants.iter().find_map(|v| v.discriminant.as_ref()) {
                bail!(discriminant, "discriminants aren't supported");
            }
            let variant = data.variants.into_iter().map(|Variant { ident, .. }| {
                let str = ident.to_string();
                quote!(Self::#ident => #str,)
            });
            quote! {
                match self {
                    #(#variant)*
                }.append(__i)
            }
        }
        Data::Union(data) => bail!(data.union_token, "unions are not supported"),
    };

    ensure!(
        generics.to_token_stream().is_empty(),
        generics,
        "generics are not supported"
    );

    Ok(quote! {
        # use ::dbus_client::__private::dbus;
        impl dbus::arg::Append for #ident {
            fn append_by_ref(&self, __i: &mut dbus::arg::IterAppend) {
                #body
            }
        }
    })
}

fn named_struct(
    named: Punctuated<syn::Field, syn::token::Comma>,
    value_signature: Option<ValueSignature>,
) -> TokenStream {
    let value_signature = value_signature.map_or_else(|| quote!("v"), |s| s.0.into_token_stream());

    let fields = named.into_iter().map(|field| {
        let fields = field.ident.as_ref().unwrap().to_string();
        let ident = field.ident;
        quote!(
            (&::dbus_client::__private::DictValue(&self.#ident))
                .append_to_dict_as_variant(#fields, __i);
        )
    });
    quote! {
        #![allow(clippy::needless_borrow)]
        use ::dbus_client::__private::AppendToDict as _;
        __i.append_dict(&"s".into(), &#value_signature.into(), |__i| {
            #(#fields)*
        })

    }
}

#[derive(FromAttr)]
struct ValueSignature(LitStr);

pub fn arg(
    DeriveInput {
        ident,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result {
    let arg_type;
    let signature;
    match data {
        Data::Enum(_) => {
            arg_type = quote!(::dbus_client::__private::dbus::arg::ArgType::String);
            signature = quote!("s".into())
        }
        _ => todo!("non enums"),
    }
    ensure!(
        generics.to_token_stream().is_empty(),
        generics,
        "generics are not supported"
    );

    Ok(quote! {
        # use ::dbus_client::__private::dbus;
        impl dbus::arg::Arg for #ident {
            const ARG_TYPE: dbus::arg::ArgType = #arg_type;
            fn signature() -> dbus::strings::Signature<'static> {
                #signature
            }
        }
    })
}

pub fn get(
    DeriveInput {
        ident,
        generics,
        data,
        ..
    }: DeriveInput,
) -> Result {
    let body;
    match data {
        Data::Enum(data) => {
            if let Some(discriminant) = data.variants.iter().find_map(|v| v.discriminant.as_ref()) {
                bail!(discriminant, "discriminants aren't supported");
            }
            let variants = data.variants.into_iter().map(|Variant { ident, .. }| {
                let str = ident.to_string();
                quote!(#str => Some(Self::#ident),)
            });
            body = quote! {
                let __s: &str = __i.get()?;
                match __s {
                    #(#variants)*
                    _ => None
                }
            };
        }
        _ => todo!("non enums"),
    }
    ensure!(
        generics.to_token_stream().is_empty(),
        generics,
        "generics are not supported"
    );

    Ok(quote! {
        # use ::dbus_client::__private::dbus;
        impl dbus::arg::Get<'_> for #ident {
            fn get(__i: &mut dbus::arg::Iter) -> Option<Self> {
                #body
            }
        }
    })
}
