table! {
    mail (id) {
        id -> Integer,
        target -> Text,
        sender -> Text,
        message -> Text,
        sent -> Timestamp,
        private -> Bool,
    }
}

table! {
    whois (nickname) {
        nickname -> Text,
        description -> Text,
    }
}

allow_tables_to_appear_in_same_query!(
    mail,
    whois,
);
