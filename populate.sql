BEGIN;

CREATE SCHEMA IF NOT EXISTS ennui;
CREATE TABLE IF NOT EXISTS ennui.help (
    helpid serial,
    title text,
    hook text[],
    description text
);

CREATE TEMPORARY TABLE thing (
    title text,
    hook text[],
    description text
);

\copy thing FROM './populate.csv' WITH (FORMAT csv);

UPDATE ennui.help
SET title = t.title,
    hook = t.hook,
    description = t.description
FROM
    thing t
WHERE
    ennui.help.title = t.title;

INSERT INTO ennui.help
(title, hook, description)
SELECT t.title, t.hook, t.description
    FROM 
        thing t
        LEFT OUTER JOIN ennui.help e
            ON e.title = t.title
    WHERE
        e.title is NULL;

DROP TABLE thing;

COMMIT;

















