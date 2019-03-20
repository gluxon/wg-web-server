table! {
    users (id) {
        id -> Integer,
        email -> Text,
        password -> Nullable<Text>,
        administrator -> Integer,
    }
}
