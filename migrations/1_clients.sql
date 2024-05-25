-- nuestros
create table if not exists clients (
    client_id       integer primary key autoincrement,

    email           varchar(255) unique not null,
    api_key         varchar(64) unique not null
);

insert into clients (email, api_key)
values ("client@email.com", "super_secret");