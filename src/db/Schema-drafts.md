

```sqlite
CREATE TABLE IF NOT EXISTS channels
(
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    discord_user_id INTEGER NOT NULL,
    channel         TEXT NOT NULL,
    UNIQUE(discord_user_id, channel) ON CONFLICT FAIL
);

CREATE TABLE IF NOT EXISTS triggers
(
    id              INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    discord_user_id INTEGER NOT NULL,
    trigger         TEXT NOT NULL,
    case_sensitive  BOOLEAN DEFAULT FALSE NOT NULL,
    regex           BOOLEAN DEFAULT FALSE NOT NULL,
    UNIQUE(discord_user_id, trigger, regex) ON CONFLICT FAIL
);

-- DROP TABLE IF EXISTS channels;
-- DROP TABLE IF EXISTS triggers;

-- DELETE FROM channels;
-- DELETE FROM triggers;

INSERT INTO channels (discord_user_id, channel) VALUES (206528846026113024, 'weest');
INSERT INTO channels (discord_user_id, channel) VALUES (206528846026113024, 'tajj');
INSERT INTO channels (discord_user_id, channel) VALUES (260457229080199179, 'weest');

INSERT INTO triggers (discord_user_id, trigger) VALUES (206528846026113024, 'is2511');
INSERT INTO triggers (discord_user_id, trigger) VALUES (206528846026113024, '@ is');
INSERT INTO triggers (discord_user_id, trigger) VALUES (260457229080199179, 'tajj');

SELECT * FROM channels WHERE discord_user_id = 206528846026113024;

SELECT discord_user_id FROM channels WHERE channel = 'weest';

SELECT trigger, case_sensitive, regex FROM triggers WHERE discord_user_id = 206528846026113024;

SELECT discord_user_id, trigger, case_sensitive, regex FROM triggers WHERE discord_user_id IN (SELECT discord_user_id FROM channels WHERE channel = 'weest');

SELECT id, trigger, case_sensitive, regex FROM triggers WHERE discord_user_id = 206528846026113024;

SELECT EXISTS(SELECT 1 FROM channels WHERE channel = 'weest');

SELECT DISTINCT channel FROM channels;
SELECT COUNT(DISTINCT channel) FROM channels;

```
