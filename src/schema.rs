table! {
    admin_logins (token, aid, login_time) {
        token -> Char,
        aid -> Char,
        login_time -> Datetime,
    }
}

table! {
    administrators (aid) {
        aid -> Char,
        password -> Char,
    }
}

table! {
    appointments (username, tid) {
        username -> Char,
        tid -> Unsigned<Bigint>,
        status -> Char,
        time -> Datetime,
    }
}

table! {
    comments (did) {
        cid -> Unsigned<Bigint>,
        username -> Char,
        did -> Char,
        comment -> Varchar,
        time -> Datetime,
    }
}

table! {
    departments (depart_name) {
        depart_name -> Char,
        information -> Varchar,
    }
}

table! {
    doctor_logins (token, did, login_time) {
        token -> Char,
        did -> Char,
        login_time -> Datetime,
    }
}

table! {
    doctors (did) {
        did -> Char,
        name -> Char,
        password -> Char,
        gender -> Char,
        birthday -> Nullable<Date>,
        department -> Char,
        rankk -> Char,
        information -> Varchar,
    }
}

table! {
    times (tid) {
        tid -> Unsigned<Bigint>,
        did -> Char,
        start_time -> Datetime,
        end_time -> Datetime,
        capacity -> Integer,
        appointed -> Integer,
    }
}

table! {
    user_logins (token, username, login_time) {
        token -> Char,
        username -> Char,
        login_time -> Datetime,
    }
}

table! {
    users (username) {
        username -> Char,
        password -> Char,
        name -> Char,
        gender -> Char,
        birthday -> Nullable<Date>,
        id_number -> Char,
        telephone -> Char,
        is_banned -> Bool,
    }
}

allow_tables_to_appear_in_same_query!(
    admin_logins,
    administrators,
    appointments,
    comments,
    departments,
    doctor_logins,
    doctors,
    times,
    user_logins,
    users,
);
