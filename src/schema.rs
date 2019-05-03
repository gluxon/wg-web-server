table! {
    peers (id) {
        id -> Integer,
        public_key -> Text,
    }
}

table! {
    users (id) {
        id -> Integer,
        email -> Text,
        password -> Nullable<Text>,
        administrator -> Integer,
    }
}

allow_tables_to_appear_in_same_query!(
    peers,
    users,
);
