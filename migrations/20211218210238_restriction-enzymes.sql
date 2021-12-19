create table restriction_enzyme (
    id uuid not null,
    name text not null unique,
    recognition_sequence text not null,
    created_at timestamptz not null,
    primary key(id)
);
