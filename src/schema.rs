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
