create table if not exists roles (
    role_id         integer primary key autoincrement,

    client_id       integer not null,

    role_name       text not null,

    foreign key(client_id) references clients (client_id)
);

create table if not exists user_roles_rel (
    user_id         integer not null,
    role_id         integer not null,

    foreign key(user_id) references users (user_id),
    foreign key(role_id) references roles (role_id)
);