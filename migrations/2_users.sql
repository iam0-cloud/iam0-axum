create table users (
    user_id         integer primary key autoincrement,

    client_id       integer not null,

    email           varchar(256) not null,
    username        varchar(256) not null,
    public_key      BLOB not null,

    unique(client_id, email),
    unique(client_id, username),
    foreign key(client_id) references clients (client_id)
);

insert into users (client_id, username, email, public_key)
values (
    (select client_id from clients where email = "client@email.com" limit 1),
    "user1",
    "user1@email.com",
    "xxx"
);

insert into users (client_id, username, email, public_key)
values (
    (select client_id from clients where email = "client@email.com" limit 1),
    "user2",
    "user2@email.com",
    "yyy"
);