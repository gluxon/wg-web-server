use rocket::http::RawStr;
use rocket::request::FromFormValue;

// A wrapper type for Option to support a custom FromFormValue impl. This version defaults
// whitespace-only fields to None.
pub struct FormOption<T>(Option<T>);

impl<T> From<FormOption<T>> for Option<T> {
    fn from(form_option: FormOption<T>) -> Self {
        form_option.0
    }
}

impl<'v, T: FromFormValue<'v>> FromFormValue<'v> for FormOption<T> {
    type Error = !;

    fn from_form_value(v: &'v RawStr) -> Result<Self, Self::Error> {
        if v.as_str().chars().all(char::is_whitespace) {
            return Ok(FormOption(None));
        }

        match T::from_form_value(v) {
            Ok(v) => Ok(FormOption(Some(v))),
            Err(_) => Ok(FormOption(None)),
        }
    }

    #[inline(always)]
    fn default() -> Option<FormOption<T>> {
        Some(FormOption(None))
    }
}
