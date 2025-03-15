macro_rules! make_error {
    ($error_name:ident $({ $($field:ident: $field_type:ident $(:: $field_type_tail:ident)*),* $(,)? })?) => {
        #[derive(Debug)]
        pub struct $error_name {
            $(
                $($field: $field_type $(:: $field_type_tail)*, )*
            )?
            message: Option<String>,
            source: Option<Box<dyn std::error::Error>>,
        }

        impl $error_name {
            #[inline]
            pub fn new($($($field: $field_type $(:: $field_type_tail)*, )*)?) -> Self {
                Self { $($($field,)*)? message: None, source: None }
            }

            #[inline]
            pub fn with_message($($($field: $field_type $(:: $field_type_tail)*, )*)? message: impl Into<String>) -> Self {
                Self { $($($field,)*)? message: Some(message.into()), source: None }
            }

            #[inline]
            pub fn with_source($($($field: $field_type $(:: $field_type_tail)*, )*)? source: Box<dyn std::error::Error>) -> Self {
                Self { $($($field,)*)? message: None, source: Some(source) }
            }

            #[inline]
            pub fn with_all($($($field: $field_type $(:: $field_type_tail)*, )*)? message: impl Into<String>, source: Box<dyn std::error::Error>) -> Self {
                Self { $($($field,)*)? message: Some(message.into()), source: Some(source) }
            }

            #[inline]
            pub fn message(&self) -> Option<&str> {
                let Some(message) = &self.message else {
                    return None;
                };
                Some(message)
            }

            $($(
                #[inline]
                pub fn $field(&self) -> $field_type $(:: $field_type_tail)* {
                    self.$field
                }
            )*)?
        }

        impl std::fmt::Display for $error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if let Some(message) = &self.message {
                    write!(f, "{message}")?;
                }

                if let Some(source) = &self.source {
                    if self.message.is_some() {
                        write!(f, ": ")?;
                    }
                    write!(f, "{:?}", source)?;
                }

                Ok(())
            }
        }

        impl std::error::Error for $error_name {
            #[inline]
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.source.as_deref()
            }
        }
    };

    ($error_name:ident $({ $($field:ident: $field_type:ident $(:: $field_type_tail:ident)*),* $(,)? })? $kind_name:ident $(impl $impl_kind_value:ident : $err_head:ident $(:: $err_tail:ident)*; )*) => {
        #[derive(Debug)]
        pub struct $error_name {
            kind: $kind_name,
            $(
                $($field: $field_type $(:: $field_type_tail)*, )*
            )?
            message: Option<String>,
            source: Option<Box<dyn std::error::Error>>,
        }

        impl $error_name {
            #[inline]
            pub fn new(kind: $kind_name $(, $($field: $field_type $(:: $field_type_tail)*)*)?) -> Self {
                Self { kind, $($($field,)*)? message: None, source: None }
            }

            #[inline]
            pub fn with_message(kind: $kind_name $(, $($field: $field_type $(:: $field_type_tail)*)*)?, message: impl Into<String>) -> Self {
                Self { kind, $($($field,)*)? message: Some(message.into()), source: None }
            }

            #[inline]
            pub fn with_source(kind: $kind_name $(, $($field: $field_type $(:: $field_type_tail)*)*)?, source: Box<dyn std::error::Error>) -> Self {
                Self { kind, $($($field,)*)? message: None, source: Some(source) }
            }

            #[inline]
            pub fn with_all(kind: $kind_name $(, $($field: $field_type $(:: $field_type_tail)*)*)?, message: impl Into<String>, source: Box<dyn std::error::Error>) -> Self {
                Self { kind, $($($field,)*)? message: Some(message.into()), source: Some(source) }
            }

            #[inline]
            pub fn kind(&self) -> $kind_name {
                self.kind
            }

            #[inline]
            pub fn message(&self) -> Option<&str> {
                let Some(message) = &self.message else {
                    return None;
                };
                Some(message)
            }

            $($(
                #[inline]
                pub fn $field(&self) -> $field_type $(:: $field_type_tail)* {
                    self.$field
                }
            )*)?
        }

        impl std::fmt::Display for $error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if let Some(message) = &self.message {
                    write!(f, "[{:?}] {message}", self.kind)?;
                } else {
                    write!(f, "{:?}", self.kind)?;
                }

                if let Some(source) = &self.source {
                    write!(f, ": {:?}", source)?;
                }

                Ok(())
            }
        }

        impl std::error::Error for $error_name {
            #[inline]
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.source.as_deref()
            }
        }

        make_error! { @impl $error_name $kind_name $(impl $impl_kind_value : $err_head $(:: $err_tail)*; )* }
    };

    ($error_name:ident $({ $($field:ident: $field_type:ident $(:: $field_type_tail:ident)*),* $(,)? })? $kind_name:ident { $($kind_value:ident),* $(,)? } $(impl $impl_kind_value:ident : $err_head:ident $(:: $err_tail:ident)*; )*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $kind_name {
            $($kind_value),*
        }

        make_error! { $error_name $({ $($field: $field_type $(:: $field_type_tail)*,)* })? $kind_name $(impl $impl_kind_value : $err_head $(:: $err_tail)*; )* }
    };

    (@impl $error_name:ident $kind_name:ident) => {};

    (@impl $error_name:ident $kind_name:ident impl $kind_value:ident : $err_head:ident $(:: $err_tail:ident)*; $($tail:tt)*) => {
        impl From<$err_head $(:: $err_tail)*> for $error_name {
            #[inline]
            fn from(value: $err_head $(:: $err_tail)*) -> Self {
                Self::with_source($kind_name::$kind_value, Box::new(value))
            }
        }

        make_error! { @impl $error_name $kind_name $($tail)*}
    };
}

make_error! {
    ReadError
    ReadErrorKind {
        IO,
        Unsupported,
        BrokenFile,
    }
    impl IO: std::io::Error;
    impl BrokenFile: IllegalDate;
    impl BrokenFile: IllegalMetaKey;
}

make_error! {
    IllegalMetaKey {
        key: u8
    }
}

make_error! {
    IllegalDate
}

make_error! {
    WriteError
    WriteErrorKind {
        IO,
        InvalidParams
    }
    impl IO: std::io::Error;
    impl InvalidParams: InvalidParams;
}

make_error! {
    InvalidParams
}
