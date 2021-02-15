BEGIN;

CREATE SCHEMA IF NOT EXISTS ennui;
CREATE TABLE IF NOT EXISTS ennui.help (
    helpid serial,
    title text,
    hook text[],
    description text
);

CREATE TABLE IF NOT EXISTS ennui.item (
    itemid serial,
    name text,
    display text,
    description text,
    hook text[],
    attributes int4[]
);

CREATE TABLE IF NOT EXISTS ennui.recipe (
    recid serial,
    itemid int,
    crafting_req int,
    exploration_req int,
    combat_req int
);

CREATE TEMPORARY TABLE thing (
    title text,
    hook text[],
    description text
);

create TEMPORARY TABLE tmpitem (
    name text,
    display text,
    description text,
    hook text[],
    attributes int4[]
);

CREATE TEMPORARY TABLE tmprecipe (
    itemname text,
    crafting_req int,
    exploration_req int,
    combat_req int
);

\copy thing FROM './populate.csv' WITH (FORMAT csv);
\copy tmpitem FROM './populate_item.csv' WITH (FORMAT csv);
\copy tmprecipe FROM './populate_recipe.csv' WITH (FORMAT csv);

UPDATE ennui.help
SET title = t.title,
    hook = t.hook,
    description = t.description
FROM
    thing t
WHERE
    ennui.help.title = t.title;

UPDATE ennui.item
SET name = t.name,
    hook = t.hook,
    description = t.description,
    attributes = t.attributes,
    display = t.display
FROM
    tmpitem t
WHERE
    ennui.item.name = t.name;


INSERT INTO ennui.help
(title, hook, description)
SELECT t.title, t.hook, t.description
    FROM 
        thing t
        LEFT OUTER JOIN ennui.help e
            ON e.title = t.title
    WHERE
        e.title is NULL;

INSERT INTO ennui.item
(name, display, description, hook, attributes)
SELECT t.name, t.display, t.description, t.hook, t.attributes
    FROM tmpitem t
    LEFT OUTER JOIN ennui.item i
        ON t.name = i.name
    WHERE i.name is NULL;

UPDATE ennui.recipe
SET combat_req = t.combat_req,
    exploration_req = t.exploration_req,
    crafting_req = t.crafting_req,
    itemid = (SELECT ei.itemid FROM ennui.item ei WHERE ei.name = t.itemname)
FROM tmprecipe t
WHERE 
    ennui.recipe.combat_req = t.combat_req and
    ennui.recipe.crafting_req = t.crafting_req and
    ennui.recipe.exploration_req = t.exploration_req;

INSERT INTO ennui.recipe
(itemid, combat_req, exploration_req, crafting_req)
SELECT (SELECT ei.itemid FROM ennui.item ei WHERE ei.name = t.itemname),
    t.combat_req, t.exploration_req, t.crafting_req
FROM tmprecipe t
LEFT OUTER JOIN ennui.recipe r
    ON 
        t.combat_req = r.combat_req and
        t.crafting_req = r.crafting_req and
        t.exploration_req = r.exploration_req
WHERE r.recid is NULL;

DROP TABLE thing;
DROP TABLE tmpitem;
DROP TABLE tmprecipe;

COMMIT;

















