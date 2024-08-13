#![warn(clippy::pedantic, /*missing_docs,*/ clippy::cargo)]
#![allow(clippy::wildcard_imports)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

use std::time::Duration;

use __private::Result;
use dbus::arg::{AppendAll, IterAppend, ReadAll};
use dbus::blocking::{BlockingSender, Connection};
use dbus::strings::{BusName, Interface, Member};
use dbus::{Message, Path};
pub use dbus_client_macros::*;

#[doc(hidden)]
pub mod __private {
    pub use dbus;
    use dbus::arg::{Append, Arg, IterAppend, Variant};
    pub type Result<T, E = dbus::Error> = std::result::Result<T, E>;

    pub trait AppendToDict {
        fn append_to_dict(&self, key: &'static str, i: &mut IterAppend);
        fn append_to_dict_as_variant(&self, key: &'static str, i: &mut IterAppend);
    }

    pub struct DictValue<T>(pub T);

    impl<T: Append + Arg> AppendToDict for &DictValue<&T> {
        fn append_to_dict(&self, key: &'static str, i: &mut IterAppend) {
            i.append_dict_entry(|i| {
                i.append(key);
                i.append(self.0);
            });
        }

        fn append_to_dict_as_variant(&self, key: &'static str, i: &mut IterAppend) {
            i.append_dict_entry(|i| {
                i.append(key);
                i.append_variant(&T::signature(), |i| i.append(self.0));
            });
        }
    }

    impl<T: Append + Arg> AppendToDict for DictValue<&Option<T>> {
        fn append_to_dict(&self, key: &'static str, i: &mut IterAppend) {
            if let Some(v) = self.0 {
                i.append_dict_entry(|i| {
                    i.append(key);
                    i.append(v);
                });
            }
        }

        fn append_to_dict_as_variant(&self, key: &'static str, i: &mut IterAppend) {
            if let Some(v) = self.0 {
                i.append_dict_entry(|i| {
                    i.append(key);
                    i.append_variant(&T::signature(), |i| i.append(v));
                });
            }
        }
    }

    impl<T: Append + Arg> DictValue<&Option<Variant<T>>> {
        pub fn append_to_dict(&self, key: &'static str, i: &mut IterAppend) {
            if let Some(v) = self.0 {
                i.append_dict_entry(|i| {
                    i.append(key);
                    i.append(v);
                });
            }
        }

        pub fn append_to_dict_as_variant(&self, key: &'static str, i: &mut IterAppend) {
            if let Some(v) = self.0 {
                i.append_dict_entry(|i| {
                    i.append(key);
                    i.append(v);
                });
            }
        }
    }
    impl<T: Append + Arg> DictValue<&Variant<T>> {
        pub fn append_to_dict(&self, key: &'static str, i: &mut IterAppend) {
            i.append_dict_entry(|i| {
                i.append(key);
                i.append(self.0);
            });
        }

        pub fn append_to_dict_as_variant(&self, key: &'static str, i: &mut IterAppend) {
            i.append_dict_entry(|i| {
                i.append(key);
                i.append(self.0);
            });
        }
    }

    #[test]
    #[allow(clippy::needless_borrow)]
    fn test() {
        use dbus::arg::{IterAppend, PropMap};
        use dbus::Message;
        let mut message = Message::new_method_call("a.b", "/a/b", "a.b", "C").unwrap();
        let i = &mut IterAppend::new(&mut message);
        i.append_dict(&"s".into(), &"v".into(), |i| {
            (&DictValue(&"a")).append_to_dict_as_variant("a", i); // calls general implementation for T: Append + Arg
            (&DictValue(&Some("b"))).append_to_dict_as_variant("b", i); // calls implementation for Option<T: Append + Arg>
            (&DictValue(&Variant("c"))).append_to_dict_as_variant("c", i); // calls implementation for Variant<T: Append + Arg>
            (&DictValue(&Some(Variant("d")))).append_to_dict_as_variant("d", i); // calls implementation for Option<Variant<T: Append + Arg>
        });

        let dict: PropMap = message.read1().unwrap();

        assert_eq!(dict.get("a").unwrap().0.signature(), "s".into());
        assert_eq!(dict.get("b").unwrap().0.signature(), "s".into());
        assert_eq!(dict.get("c").unwrap().0.signature(), "s".into());
        assert_eq!(dict.get("d").unwrap().0.signature(), "s".into());
    }
}

pub trait CommonDestination {
    const DESTINATION: &'static str;
}

pub trait CommonPath {
    const PATH: &'static str;
}

pub trait CommonlySession {}
pub trait CommonlySystem {}

pub enum MaybeOwned<'a, T> {
    Owned(T),
    Borrowed(&'a T),
}

impl<T> AsRef<T> for MaybeOwned<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            MaybeOwned::Owned(t) => t,
            MaybeOwned::Borrowed(t) => t,
        }
    }
}

impl<T> From<T> for MaybeOwned<'static, T> {
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'a, T> From<&'a T> for MaybeOwned<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Borrowed(value)
    }
}

pub trait DbusObject<'a>: Sized {
    fn new(
        conn: impl Into<MaybeOwned<'a, Connection>>,
        destination: impl Into<BusName<'a>>,
        path: impl Into<Path<'a>>,
        timeout: Duration,
    ) -> Self;

    fn connect(conn: impl Into<MaybeOwned<'a, Connection>>, timeout: Duration) -> Self
    where
        Self: CommonDestination + CommonPath,
    {
        Self::new(conn, Self::DESTINATION, Self::PATH, timeout)
    }

    fn with_destination(
        conn: impl Into<MaybeOwned<'a, Connection>>,
        destination: impl Into<BusName<'a>>,
        timeout: Duration,
    ) -> Self
    where
        Self: CommonPath,
    {
        Self::new(conn, destination, Self::PATH, timeout)
    }

    fn with_path(
        conn: impl Into<MaybeOwned<'a, Connection>>,
        path: impl Into<Path<'a>>,
        timeout: Duration,
    ) -> Self
    where
        Self: CommonDestination,
    {
        Self::new(conn, Self::DESTINATION, path, timeout)
    }

    fn session(timeout: Duration) -> Result<Self>
    where
        Self: CommonDestination + CommonPath + CommonlySession,
        'a: 'static,
    {
        let conn = Connection::new_session()?;
        Ok(Self::connect(conn, timeout))
    }

    fn system(timeout: Duration) -> Result<Self>
    where
        Self: CommonDestination + CommonPath + CommonlySystem,
        'a: 'static,
    {
        let conn = Connection::new_system()?;
        Ok(Self::connect(conn, timeout))
    }

    fn connection(this: &Self) -> &Connection;
    fn destination(this: &Self) -> &BusName<'a>;
    fn path(this: &Self) -> &Path<'a>;
    fn timeout(this: &Self) -> Duration;

    fn sub_object<T: DbusObject<'a>>(this: &'a Self, path: impl Into<Path<'a>>) -> T {
        T::new(
            DbusObject::connection(this),
            DbusObject::destination(this),
            path,
            DbusObject::timeout(this),
        )
    }

    fn method_call<'b, R: ReadAll>(
        &'a self,
        interface: impl Into<Interface<'b>>,
        member: impl Into<Member<'b>>,
        args: impl AppendAll,
    ) -> Result<R> {
        let mut msg = Message::method_call(
            Self::destination(self),
            Self::path(self),
            &interface.into(),
            &member.into(),
        );
        args.append(&mut IterAppend::new(&mut msg));
        let r = Self::connection(self).send_with_reply_and_block(msg, Self::timeout(self))?;
        Ok(R::read(&mut r.iter_init())?)
    }
}

// impl<'b, T: for<'a> Get<'a>> GetWithConnection<'b> for T {
//     fn get(_: &impl DbusObject<'b>, i: &mut Iter<'_>) -> Option<Self> {
//         i.get()
//     }
// }

// pub trait GetWithConnection<'a>: Sized {
//     fn get(parent: &'a impl DbusObject<'a>, i: &mut Iter<'_>) ->
// Option<Self>;     fn read(parent: &'a impl DbusObject<'a>, i: &mut Iter<'_>)
// -> Result<Self, TypeMismatchError>     where
//         Self: Arg,
//     {
//         let it = Self::get(parent, i).ok_or_else(|| TypeMismatchError {
//             expected: Self::ARG_TYPE,
//             found: i.arg_type(),
//         })?;
//         i.next();
//         Ok(it)
//     }
// }

// pub trait ReadAllWithConnection<'a>: Sized {
//     fn read<'b>(parent: &'a impl DbusObject<'a>, i: &mut Iter<'b>) ->
// Result<Self, TypeMismatchError>; }

// // impl ReadAllWithConnection<'_> for () {
// //     fn read(_: &Connection, _: &mut Iter) -> Result<Self,
// TypeMismatchError> // {         Ok(())
// //     }
// // }

// forr! { tuples($tuple:ident) in idents(i, 26) $*
//     forr!{ casing($field:s, $Gen:C) in [$tuple] $:
//         impl<'a, $($Gen: Arg + GetWithConnection<'a>,)*>
// ReadAllWithConnection<'a> for ($($Gen,)*) {             #[allow(unused)]
//             fn read<'b>(parent: &'a impl DbusObject<'a>, i: &mut Iter<'b>) ->
// Result<Self, TypeMismatchError>  {                 $(let $field =
// $Gen::read(parent, i)?;)*                 Ok(($($field,)*))
//             }
//         }
//     }
// }

// // TODO consider making the dbus crates one public instead.
// /// Error struct to indicate a D-Bus argument type mismatch.
// ///
// /// Might be returned from `iter::read()`.
// #[derive(Clone, Copy, Debug, PartialEq, Eq)]
// pub struct TypeMismatchError {
//     expected: ArgType,
//     found: ArgType,
//     // TODO currently not public position: u32,
// }

// impl TypeMismatchError {
//     /// The [`ArgType`] we were trying to read, but failed
//     #[must_use]
//     pub fn expected_arg_type(&self) -> ArgType {
//         self.expected
//     }

//     /// The [`ArgType`] we should have been trying to read, if we wanted the
//     /// read to succeed
//     #[must_use]
//     pub fn found_arg_type(&self) -> ArgType {
//         self.found
//     }

//     // TODO currently not public
//     // /// At what argument was the error found?
//     // ///
//     // /// Returns 0 for first argument, 1 for second argument, etc.
//     // pub fn pos(&self) -> u32 {
//     //     self.position
//     // }
// }

// impl error::Error for TypeMismatchError {
//     fn description(&self) -> &str {
//         "D-Bus argument type mismatch"
//     }

//     fn cause(&self) -> Option<&dyn error::Error> {
//         None
//     }
// }

// impl From<TypeMismatchError> for dbus::Error {
//     fn from(t: TypeMismatchError) -> dbus::Error {
//         dbus::Error::new_custom("org.freedesktop.DBus.Error.Failed",
// &format!("{}", t))     }
// }

// impl Display for TypeMismatchError {
//     fn fmt(&self, f: &mut Formatter) -> fmt::Result {
//         write!(
//             f,
//             "D-Bus argument type mismatch at expected {}, found {}",
//             // self.position,
//             self.expected.as_str(),
//             if self.expected == self.found {
//                 "same but still different somehow"
//             } else {
//                 self.found.as_str()
//             }
//         )
//     }
// }

// pub trait Properties {
//     fn get<R0: for<'b> GetWithConnection<'b> + Arg + 'static>(
//         &self,
//         interface_name: &str,
//         property_name: &str,
//     ) -> Result<R0, dbus::Error>;
//     fn get_all(&self, interface_name: &str) -> Result<arg::PropMap,
// dbus::Error>;     fn set<I2: arg::Arg + arg::Append>(
//         &self,
//         interface_name: &str,
//         property_name: &str,
//         value: I2,
//     ) -> Result<(), dbus::Error>;
// }
