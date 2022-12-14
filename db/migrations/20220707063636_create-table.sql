CREATE TABLE user (
    uid INTEGER PRIMARY KEY NOT NULL,
    username TEXT NOT NULL UNIQUE,
    password TEXT NOT NULL
);

CREATE TABLE announcement (
    aid INTEGER PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL
);

CREATE TABLE post (
    pid INTEGER PRIMARY KEY NOT NULL,
    author TEXT NOT NULL,
    time TEXT NOT NULL,
    content TEXT NOT NULL
);

CREATE TABLE comment (
    cid INTEGER PRIMARY KEY NOT NULL,
    author TEXT NOT NULL,
    time TEXT NOT NULL,
    content TEXT NOT NULL,
    pid INTEGER NOT NULL
);
