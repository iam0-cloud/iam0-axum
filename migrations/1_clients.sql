-- nuestros
create table if not exists clients (
    client_id       integer primary key autoincrement,
    email           varchar(255) unique not null
);

-- del cliente
create table if not exists users (
    client_id       integer,

    user_id         integer primary key autoincrement,
    email           varchar(255) unique not null,

    foreign key(client_id) references clients(client_id)     
);

create table if not exists user_role_rel (
    client_id       integer not null,

    user_id         integer not null,
    role_id         integer not null,

    primary key(client_id, user_id, role_id),
    foreign key(client_id) references clients(client_id),
    foreign key(user_id) references users(user_id),
    foreign key(role_id) references roles(role_id)
);

create table if not exists roles (
    client_id       integer not null,

    role_id         integer primary key autoincrement,
    name            text not null
);