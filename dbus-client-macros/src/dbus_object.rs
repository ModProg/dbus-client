use syn::Attribute;

use super::*;

pub fn dbus_object(
    Object {
        attributes,
        name,
        dest,
        path,
        session,
        system,
        interfaces,
    }: Object,
) -> Result {
    let path = path.as_slice();
    let dest = dest.as_slice();

    let interfaces = interfaces.into_iter().map(|i| i.expand(&name));

    let mut extra_traits = TokenStream::new();

    if session {
        quote! {
            impl ::dbus_client::CommonlySession for #name<'_> {}
        }
        .to_tokens(&mut extra_traits);
    }
    if system {
        quote! {
            impl ::dbus_client::CommonlySystem for #name<'_> {}
        }
        .to_tokens(&mut extra_traits);
    }

    Ok(quote! {
        # use dbus_client::__private::dbus::blocking::Connection;
        # use dbus_client::__private::dbus::strings::{BusName, Path, Signature};
        # use dbus_client::__private::dbus::arg::{self, Arg, ArgType, Get, Append, IterAppend};
        # use dbus_client::__private::dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
        # use std::time::Duration;
        # use dbus_client::{CommonDestination, CommonPath, DbusObject, MaybeOwned};

        #(#attributes)*
        pub struct #name<'a> {
            connection: MaybeOwned<'a, Connection>,
            destination: BusName<'a>,
            path: Path<'a>,
            timeout: Duration,
        }
        #[allow(non_snake_case)]
        const _: () = {

        impl std::fmt::Debug for #name<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
                f.debug_struct(stringify!(#name))
                    .field("destination", &self.destination)
                    .field("path", &self.path)
                    .field("timeout", &self.timeout)
                    .finish()
            }
        }

        impl<'a> DbusObject<'a> for #name<'a> {
            fn new(
                connection: impl Into<MaybeOwned<'a, Connection>>,
                destination: impl Into<BusName<'a>>,
                path: impl Into<Path<'a>>,
                timeout: Duration
            ) -> Self {
                Self {
                    connection: connection.into(),
                    destination: destination.into(),
                    path: path.into(),
                    timeout
                }
            }

            fn connection(this: &Self) -> &Connection {
                AsRef::as_ref(&this.connection)
            }

            fn destination(this: &Self) -> &BusName<'a> {
                &this.destination
            }

            fn path(this: &Self) -> &Path<'a> {
                &this.path
            }
            fn timeout(this: &Self) -> Duration {
                this.timeout
            }
        }

        #(impl CommonDestination for #name<'_> {
            const DESTINATION: &'static str = #dest;
        })*

        #(impl CommonPath for #name<'_> {
            const PATH: &'static str = #path;
        })*

        #extra_traits

        impl Properties for #name<'_> {

            fn get<R0: for<'b> Get<'b> + 'static>(&self, interface_name: &str, property_name: &str) -> Result<R0, dbus::Error> {
                DbusObject::method_call(self, "org.freedesktop.DBus.Properties", "Get", (interface_name, property_name, ))
                    .and_then(|r: (arg::Variant<R0>, )| Ok((r.0).0, ))
            }

            fn get_all(&self, interface_name: &str) -> Result<arg::PropMap, dbus::Error> {
                DbusObject::method_call(self, "org.freedesktop.DBus.Properties", "GetAll", (interface_name, ))
                    .and_then(|r: (arg::PropMap, )| Ok(r.0, ))
            }

            fn set<I2: arg::Arg + arg::Append>(&self, interface_name: &str, property_name: &str, value: I2) -> Result<(), dbus::Error> {
                DbusObject::method_call(self, "org.freedesktop.DBus.Properties", "Set", (interface_name, property_name, arg::Variant(value), ))
            }
        }

        impl Arg for #name<'_> {
            const ARG_TYPE: ArgType = <Path as Arg>::ARG_TYPE;

            fn signature() -> Signature<'static> {
                <Path as Arg>::signature()
            }
        }

        impl Append for #name<'_> {
            fn append_by_ref(&self, __i: &mut IterAppend<'_>) {
                self.path.append_by_ref(__i);
            }
        }

        #(#interfaces)*
        };
    })
}

pub struct Object {
    attributes: Vec<Attribute>,
    name: Ident,
    // "([a-zA-Z_-][a-zA-Z0-9_-]*\.)+[a-zA-Z_-][a-zA-Z0-9_-]*"
    dest: Option<LitStr>,
    // "(/[a-zA-Z0-9_]+)+"
    path: Option<LitStr>,
    session: bool,
    system: bool,
    interfaces: Vec<InterfaceImpl>,
}

impl Parse for Object {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = Attribute::parse_outer(input)?;
        let name = input.parse()?;
        let mut dest = None;
        let mut path = None;
        let mut session = None;
        let mut system = None;
        if input.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let properties = Punctuated::<ExtraTraits, Token![,]>::parse_terminated(&content)?;
            for property in properties {
                match property {
                    ExtraTraits::Destination(new) => {
                        if let Some(old) = dest {
                            let mut error =
                                syn::Error::new_spanned(new, "destination already defined");
                            error.combine(syn::Error::new_spanned(
                                old,
                                "destination was defined here",
                            ));
                            return Err(error);
                        } else {
                            dest = Some(new);
                        }
                    }
                    ExtraTraits::Path(new) => {
                        if let Some(old) = path {
                            let mut error = syn::Error::new_spanned(new, "path already defined");
                            error.combine(syn::Error::new_spanned(old, "path was defined here"));
                            return Err(error);
                        } else {
                            path = Some(new);
                        }
                    }
                    ExtraTraits::Session(new) => {
                        if let Some(old) = session {
                            let mut error = syn::Error::new_spanned(new, "session was already set");
                            error.combine(syn::Error::new_spanned(
                                old,
                                "session was already set here",
                            ));
                            return Err(error);
                        } else {
                            session = Some(new);
                        }
                    }
                    ExtraTraits::System(new) => {
                        if let Some(old) = system {
                            let mut error = syn::Error::new_spanned(new, "system was already set");
                            error.combine(syn::Error::new_spanned(
                                old,
                                "system was already set here",
                            ));
                            return Err(error);
                        } else {
                            system = Some(new);
                        }
                    }
                }
            }
        }

        let mut interfaces = Vec::new();

        while !input.is_empty() {
            interfaces.push(input.parse()?);
        }

        Ok(Self {
            attributes,
            name,
            dest,
            path,
            interfaces,
            session: session.is_some(),
            system: system.is_some(),
        })
    }
}

mod property {
    use syn::custom_keyword;

    custom_keyword!(session);
    custom_keyword!(system);
}

enum ExtraTraits {
    Destination(LitStr),
    Path(LitStr),
    Session(property::session),
    System(property::system),
}

impl Parse for ExtraTraits {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let la = input.lookahead1();
        Ok(if la.peek(property::session) {
            Self::Session(input.parse()?)
        } else if la.peek(property::system) {
            Self::System(input.parse()?)
        } else if la.peek(LitStr) {
            let str: LitStr = input.parse()?;
            let value = str.value();
            if let Some(value) = value.strip_prefix("/") {
                if value.split("/").any(|e| {
                    e.is_empty()
                        || e.contains(|c: char| !matches!(c, 'a'..='z'|'A'..='Z'|'0'..='9'|'_'))
                }) {
                    return Err(error_message!(
                        str,
                        "Path starts with `/`, must only consist of `/` separated non zero length \
                         segments containing only ASCII letters, numbers or `_`, and must not end \
                         in `/`. REGEX: `(/[a-zA-Z0-9_]+)+`";
                         info="Destinations must not start with `/`."
                    )
                    .into());
                } else {
                    Self::Path(str)
                }
            } else {
                if value.split(".").any(|e| {
                    !e.starts_with(|c: char| matches!(c, 'a'..='z'|'A'..='Z'|'_'|'-'))
                        || e.contains(|c: char| !matches!(c, 'a'..='z'|'A'..='Z'|'0'..='9'|'_'|'-'))
                }) {
                    return Err(error_message!(
                        str,
                        "Destinations must only consist of `.` separated non zero length segments \
                         containing only ASCII letters, numbers, `-` or `_` that do not start \
                         with a number. Destinations may neither start nor end in `.`. REGEX: \
                         `([a-zA-Z_-][a-zA-Z0-9_-]*\\.)+[a-zA-Z_-][a-zA-Z0-9_-]*`";
                         info="Paths must start with `/`."
                    )
                    .into());
                } else {
                    Self::Destination(str)
                }
            }
        } else {
            return Err(la.error());
        })
    }
}

enum InterfaceImpl {
    Anonymous(Interface),
    Named(Ident),
}

impl Parse for InterfaceImpl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek(Ident) {
            let name = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::Named(name))
        } else {
            input.parse().map(Self::Anonymous)
        }
    }
}

impl InterfaceImpl {
    fn expand(self, struct_name: &Ident) -> TokenStream {
        match self {
            InterfaceImpl::Anonymous(Interface { name, members }) => {
                let members = members.into_iter().map(|m| m.expand(&name));
                quote!(impl #struct_name<'_> { #(#members)* })
            }
            InterfaceImpl::Named(name) => quote! {
                impl #name for #struct_name { }
            },
        }
    }
}

struct Interface {
    // "([a-zA-Z_][a-aA-Z0-9_]*\.)+[a-zA-Z_][a-aA-Z0-9_]*"
    name: LitStr,
    members: Vec<Member>,
}

impl Parse for Interface {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;

        let content;
        braced!(content in input);

        let mut members = Vec::new();
        while !content.is_empty() {
            members.push(content.parse()?);
        }
        Ok(Self { name, members })
    }
}

enum Member {
    Property(Property),
    Method(Method),
}
impl Member {
    fn expand(self, interface: &LitStr) -> TokenStream {
        let (Member::Property(Property { ty, .. }) | Member::Method(Method { output: ty, .. })) =
            &self;
        let mut transformer = ty.transformer();
        match self {
            Member::Property(Property {
                attributes,
                mutability,
                name,
                ty,
            }) => {
                let names = name.to_string();
                let set = if mutability.is_some() {
                    let set = format_ident!("set_{name}");
                    quote! {
                        # use dbus_client::__private::dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
                        # use ::dbus_client::__private::Result;

                        #(#attributes)*
                        pub fn #set(&self, value: #ty) -> Result<()> {
                            Properties::set(self, #interface, #names, value)
                        }
                    }
                } else {
                    quote!()
                };

                let get = format_ident!("get_{name}");
                quote! {
                    # use dbus_client::__private::dbus::blocking::stdintf::org_freedesktop_dbus::Properties;
                    # use ::dbus_client::__private::Result;

                    #set

                    #(#attributes)*
                    pub fn #get(&self) -> Result<#ty> {
                        Properties::get(self, #interface, #names) #transformer
                    }
                }
            }
            Member::Method(Method {
                attributes,
                name,
                args,
                output,
            }) => {
                let names = name.to_string();
                if !matches!(output, Type::Struct(_) | Type::Empty) {
                    transformer = Some(quote!(.map(|(v,)|v) #transformer));
                }
                let param_names = args.iter().map(|a| &a.name);
                quote! {
                    # use ::dbus_client::__private::Result;
                    # use ::dbus_client::DbusObject;

                    #(#attributes)*
                    pub fn #name(&self, #(#args),*) -> Result<#output> {
                        DbusObject::method_call(self, #interface, #names, (#(#param_names,)*)) #transformer
                    }
                }
            }
        }
    }
}

impl Parse for Member {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = Attribute::parse_outer(input)?;
        let mutability: Option<_> = input.parse()?;
        let name = input.parse()?;

        let la = input.lookahead1();
        if mutability.is_some() || la.peek(Token![:]) {
            input.parse::<Token![:]>()?;
            let ty = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Self::Property(Property {
                attributes,
                mutability,
                name,
                ty,
            }))
        } else if la.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let mut args = Vec::new();
            while !content.is_empty() {
                args.push(content.parse()?);
            }

            let output = if input.parse::<Token![->]>().is_ok() {
                input.parse()?
            } else {
                Type::Empty
            };

            input.parse::<Token![;]>()?;

            Ok(Self::Method(Method {
                attributes,
                name,
                args,
                output,
            }))
        } else {
            Err(la.error())
        }
    }
}

struct Property {
    mutability: Option<Token![mut]>,
    name: Ident,
    ty: Type,
    attributes: Vec<Attribute>,
}

mod types {
    use syn::custom_keyword;

    custom_keyword!(y);
    custom_keyword!(b);
    custom_keyword!(n);
    custom_keyword!(q);
    custom_keyword!(i);
    custom_keyword!(u);
    custom_keyword!(d);
    custom_keyword!(h);
    custom_keyword!(s);
    custom_keyword!(o);
    custom_keyword!(g);
    custom_keyword!(a);
    custom_keyword!(v);
}

enum SimpleType {
    U8,
    Bool,
    I16,
    U16,
    I32,
    U32,
    F64,
    File,
    String,
    Path,
    Signature,
}

impl ToTokens for SimpleType {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            SimpleType::U8 => quote!(u8),
            SimpleType::Bool => quote!(bool),
            SimpleType::I16 => quote!(i16),
            SimpleType::U16 => quote!(u16),
            SimpleType::I32 => quote!(i32),
            SimpleType::U32 => quote!(u32),
            SimpleType::F64 => quote!(f64),
            SimpleType::File => quote!(::std::fs::File),
            SimpleType::String => quote!(String),
            SimpleType::Path => quote!(::dbus_client::__private::dbus::strings::Path),
            SimpleType::Signature => quote!(::dbus_client::__private::dbus::strings::Signature),
        }
        .to_tokens(tokens)
    }
}

impl Parse for SimpleType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use types::*;
        let la = input.lookahead1();
        let simple = if la.peek(y) {
            SimpleType::U8
        } else if la.peek(b) {
            SimpleType::Bool
        } else if la.peek(n) {
            SimpleType::I16
        } else if la.peek(q) {
            SimpleType::U16
        } else if la.peek(i) {
            SimpleType::I32
        } else if la.peek(u) {
            SimpleType::U32
        } else if la.peek(d) {
            SimpleType::F64
        } else if la.peek(h) {
            SimpleType::File
        } else if la.peek(s) {
            SimpleType::String
        } else if la.peek(o) {
            SimpleType::Path
        } else if la.peek(g) {
            SimpleType::Signature
        } else {
            return Err(la.error());
        };
        input.parse::<Ident>()?;
        Ok(simple)
    }
}

enum Type {
    Variant,
    Rust(Path),
    Object(#[allow(unused)] Token![@], Path),
    Simple(SimpleType),
    Struct(Vec<Type>),
    Array(Box<Type>),
    Map(SimpleType, Box<Type>),
    Empty,
}
impl Type {
    fn transformer(&self) -> Option<TokenStream> {
        match self {
            Type::Object(..) => Some(
                quote!(.map(|path: ::dbus_client::__private::dbus::strings::Path| ::dbus_client::DbusObject::sub_object(self, path))),
            ),
            Type::Struct(_) => todo!(),
            Type::Array(ty) => ty
                .transformer()
                .map(|sub| quote!(.map(|value: Vec<_>| value.into_iter()#sub.collect()))),
            Type::Map(..) => todo!(),
            _ => None,
        }
    }
}

impl ToTokens for Type {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Type::Variant => quote!(
                ::dbus_client::__private::dbus::arg::Variant<
                    Box<dyn ::dbus_client::__private::dbus::arg::RefArg + 'static>,
                >
            )
            .to_tokens(tokens),
            Type::Rust(path) => path.to_tokens(tokens),
            Type::Simple(simple) => simple.to_tokens(tokens),
            Type::Struct(t) => quote!((#(#t,)*)).to_tokens(tokens),
            Type::Array(t) => quote!(Vec<#t>).to_tokens(tokens),
            Type::Map(k, v) => quote!(::std::collections::HashMap<#k, #v>).to_tokens(tokens),
            Type::Empty => quote!(()).to_tokens(tokens),
            Type::Object(_, path) => path.to_tokens(tokens),
        }
    }
}

impl Parse for Type {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        use types::*;
        let la = input.lookahead1();
        Ok(if la.peek(Paren) {
            let content;
            parenthesized!(content in input);
            let mut types = Vec::new();
            while !content.is_empty() {
                types.push(content.parse()?);
            }
            Type::Struct(types)
        } else if la.peek(a) {
            input.parse::<a>()?;
            if input.peek(Brace) {
                let content;
                braced!(content in input);
                Type::Map(content.parse()?, content.parse()?)
            } else {
                Type::Array(input.parse()?)
            }
        } else if la.peek(v) {
            input.parse::<v>()?;
            Type::Variant
        } else if la.peek(Token![@]) {
            Type::Object(input.parse()?, input.parse()?)
        } else {
            let simple = Type::Simple(if la.peek(y) {
                SimpleType::U8
            } else if la.peek(b) {
                SimpleType::Bool
            } else if la.peek(n) {
                SimpleType::I16
            } else if la.peek(q) {
                SimpleType::U16
            } else if la.peek(i) {
                SimpleType::I32
            } else if la.peek(u) {
                SimpleType::U32
            } else if la.peek(d) {
                SimpleType::F64
            } else if la.peek(h) {
                SimpleType::File
            } else if la.peek(s) {
                SimpleType::String
            } else if la.peek(o) {
                SimpleType::Path
            } else if la.peek(g) {
                SimpleType::Signature
            } else if input.peek(Ident) || input.peek(Token![::]) {
                return Ok(Type::Rust(input.parse()?));
            } else {
                return Err(la.error());
            });
            input.parse::<Ident>()?;
            simple
        })
    }
}

struct Method {
    attributes: Vec<Attribute>,
    name: Ident,
    args: Vec<Arg>,
    output: Type,
}

struct Arg {
    name: Ident,
    ty: Type,
}

impl Parse for Arg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse()?;
        input.parse::<Token![:]>()?;
        Ok(Self {
            name,
            ty: input.parse()?,
        })
    }
}

impl ToTokens for Arg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        let Self { name, ty } = self;

        quote!(#name: #ty).to_tokens(tokens);
    }
}
