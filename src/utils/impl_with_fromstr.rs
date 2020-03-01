use rocket::http::RawStr;
use std::str::FromStr;
use std::str::Utf8Error;

pub enum FormInputErrorError<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    Utf8Error(Utf8Error),
    FromStr(<T as FromStr>::Err),
}

impl<T> std::fmt::Display for FormInputErrorError<T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            FormInputErrorError::Utf8Error(error) => error.fmt(f),
            FormInputErrorError::FromStr(error) => error.fmt(f),
        }
    }
}

pub struct FormInputError<'v, T>
where
    T: FromStr,
    <T as FromStr>::Err: std::fmt::Display,
{
    pub error: FormInputErrorError<T>,
    pub input: &'v RawStr,
}

pub type FormInputResult<'v, T> = Result<T, FormInputError<'v, T>>;

// The following macro is modified from a version found within Rocket. As such, it is licensed under
// the same license as Rocket. The original version simply returned the original RawStr in Result
// error variant. This version returns the FromStr error and the original string.
//
// https://github.com/SergioBenitez/Rocket/blob/3e4f845/core/lib/src/request/form/from_form_value.rs#L230
//
// The MIT License (MIT)
// Copyright (c) 2016-2019 Sergio Benitez

#[macro_export]
macro_rules! impl_with_fromstr_with_error {
    ($($T:ident),+) => ($(
        impl<'v> rocket::request::FromFormValue<'v> for $T {
            type Error = crate::utils::FormInputError<'v, $T>;

            #[inline(always)]
            fn from_form_value(v: &'v rocket::http::RawStr) -> Result<Self, Self::Error> {
                let decoded = v.url_decode().map_err(|error|
                    Self::Error {
                        error: crate::utils::FormInputErrorError::Utf8Error(error),
                        input: v
                    })?;

                $T::from_str(&decoded)
                    .map_err(|error| Self::Error {
                        error: crate::utils::FormInputErrorError::FromStr(error),
                        input: v,
                    })
            }
        }
    )+)
}
