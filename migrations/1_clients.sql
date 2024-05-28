-- nuestros
create table if not exists clients (
    client_id       integer primary key autoincrement,

    company_name    varchar(256) not null,
    email           varchar(256) unique not null,
    api_key         varchar(64) unique not null
);

insert into clients (company_name, email, api_key)
values ("companyTM", "client@email.com", "super_secret");

insert into clients (company_name, email, api_key)
values ("uwuSL", "client2@email.com", "super_secret2");