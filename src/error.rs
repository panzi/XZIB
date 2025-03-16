#[macro_export]
macro_rules! make_error {
    ($error_name:ident; $(struct $inner_name:ident { $($field:ident: $field_type:ident $(:: $field_type_tail:ident)*),* $(,)? })? enum $kind_name:ident { $($kind_value:ident),* $(,)? } $(impl $impl_kind_value:ident : $err_head:ident $(:: $err_tail:ident)*; )*) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum $kind_name {
            $($kind_value),*
        }

        make_error! { @declare $error_name; $(struct $inner_name)? { kind: $kind_name, $( $($field: $field_type $(:: $field_type_tail)*,)* )? } enum $kind_name $(impl $impl_kind_value : $err_head $(:: $err_tail)*; )* }
    };

    ($error_name:ident; $(struct $inner_name:ident { $($field:ident: $field_type:ident $(:: $field_type_tail:ident)*),* $(,)? })?) => {
        make_error! { @declare $error_name; $(struct $inner_name)? { $( $($field: $field_type $(:: $field_type_tail)*,)* )? } }
    };

    (@declare $error_name:ident; struct $inner_name:ident $({ $($field:ident: $field_type:ident $(:: $field_type_tail:ident)*),* $(,)? })? $(enum $kind_name:ident $(impl $impl_kind_value:ident : $err_head:ident $(:: $err_tail:ident)*; )*)?) => {
        // Fields are put into boxed inner struct, so that the Result will only have one pointer size data for the error case.
        #[derive(Debug)]
        struct $inner_name {
            $(
                $($field: $field_type $(:: $field_type_tail)*, )*
            )?
            message: Option<String>,
            source: Option<Box<dyn std::error::Error>>,
        }

        #[derive(Debug)]
        pub struct $error_name {
            inner: Box<$inner_name>,
        }

        impl $error_name {
            #[inline]
            pub fn new($($($field: $field_type $(:: $field_type_tail)*),*)?) -> Self {
                Self { inner: Box::new($inner_name { $($($field,)*)? message: None, source: None }) }
            }

            #[inline]
            pub fn with_message($($($field: $field_type $(:: $field_type_tail)*, )*)? message: impl Into<String>) -> Self {
                Self { inner: Box::new($inner_name { $($($field,)*)? message: Some(message.into()), source: None }) }
            }

            #[inline]
            pub fn with_source($($($field: $field_type $(:: $field_type_tail)*, )*)? source: Box<dyn std::error::Error>) -> Self {
                Self { inner: Box::new($inner_name { $($($field,)*)? message: None, source: Some(source) }) }
            }

            #[inline]
            pub fn with_all($($($field: $field_type $(:: $field_type_tail)*, )*)? message: impl Into<String>, source: Box<dyn std::error::Error>) -> Self {
                Self { inner: Box::new($inner_name { $($($field,)*)? message: Some(message.into()), source: Some(source) }) }
            }

            #[inline]
            pub fn message(&self) -> Option<&str> {
                let Some(message) = &self.inner.message else {
                    return None;
                };
                Some(message)
            }

            $($(
                #[inline]
                pub fn $field(&self) -> $field_type $(:: $field_type_tail)* {
                    self.inner.$field
                }
            )*)?
        }

        make_error!(@display @inner $error_name $(enum $kind_name)?);

        impl std::error::Error for $error_name {
            #[inline]
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.inner.source.as_deref()
            }
        }

        $(make_error! { @impl $error_name enum $kind_name $(impl $impl_kind_value : $err_head $(:: $err_tail)*; )* })?
    };

    (@declare $error_name:ident; $({ $($field:ident: $field_type:ident $(:: $field_type_tail:ident)*),* $(,)? })? $(enum $kind_name:ident $(impl $impl_kind_value:ident : $err_head:ident $(:: $err_tail:ident)*; )*)?) => {
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
            pub fn new($($($field: $field_type $(:: $field_type_tail)*),*)?) -> Self {
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

        make_error!(@display $error_name $(enum $kind_name)?);

        impl std::error::Error for $error_name {
            #[inline]
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                self.source.as_deref()
            }
        }

        $(make_error! { @impl $error_name enum $kind_name $(impl $impl_kind_value : $err_head $(:: $err_tail)*; )* })?
    };

    (@get @inner $obj:ident $field:ident) => {
        $obj.inner.$field
    };

    (@get $obj:ident $field:ident) => {
        $obj.$field
    };

    (@display $(@$flag:ident)? $error_name:ident) => {
        impl std::fmt::Display for $error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if let Some(message) = &make_error!(@get $(@$flag)? self message) {
                    message.fmt(f)?;
                }

                if let Some(source) = &make_error!(@get $(@$flag)? self source) {
                    write!(f, ": {:?}", source)?;
                }

                Ok(())
            }
        }
    };

    (@display $(@$flag:ident)? $error_name:ident $(enum $kind_name:ident)?) => {
        impl std::fmt::Display for $error_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                if let Some(message) = &make_error!(@get $(@$flag)? self message) {
                    write!(f, "[{:?}] {message}", make_error!(@get $(@$flag)? self kind))?;
                } else {
                    write!(f, "{:?}", make_error!(@get $(@$flag)? self kind))?;
                }

                if let Some(source) = &make_error!(@get $(@$flag)? self source) {
                    write!(f, ": {:?}", source)?;
                }

                Ok(())
            }
        }
    };

    (@impl $error_name:ident enum $kind_name:ident) => {};

    (@impl $error_name:ident enum $kind_name:ident impl $kind_value:ident : $err_head:ident $(:: $err_tail:ident)*; $($tail:tt)*) => {
        impl From<$err_head $(:: $err_tail)*> for $error_name {
            #[inline]
            fn from(value: $err_head $(:: $err_tail)*) -> Self {
                Self::with_source($kind_name::$kind_value, Box::new(value))
            }
        }

        make_error! { @impl $error_name enum $kind_name $($tail)* }
    };
}

make_error! {
    ReadError;
    struct ReadErrorInner {}
    enum ReadErrorKind {
        IO,
        Unsupported,
        BrokenFile,
    }
    impl IO: std::io::Error;
    impl BrokenFile: IllegalDate;
    impl BrokenFile: IllegalMetaKey;
}

make_error! {
    IllegalMetaKey;
    struct IllegalMetaKeyInner {
        key: u8
    }
}

make_error! {
    IllegalDate;
    struct IllegalDateInner {}
}

make_error! {
    WriteError;
    struct WriteErrorInner {}
    enum WriteErrorKind {
        IO,
        InvalidParams
    }
    impl IO: std::io::Error;
    impl InvalidParams: InvalidParams;
}

make_error! {
    InvalidParams;
    struct InvalidParamsInner {}
}
