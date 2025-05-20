-- -----------------------------------------------
-- -----------------------------------------------
-- DROP view derive_accounts;
-- -----------------------------------------------
-- -----------------------------------------------
DROP VIEW mv_all_mints;
DROP VIEW mv_all_burns;
DROP VIEW mv_mint_per_month;
DROP VIEW mv_burn_per_month;
DROP VIEW mv_supply;

-- -----------------------------------------------
-- -----------------------------------------------
-- mv_all_mints
-- -----------------------------------------------
-- -----------------------------------------------
CREATE MATERIALIZED VIEW mv_all_mints
    ENGINE = ReplacingMergeTree()
    ORDER BY (instruction_id)
AS
SELECT b.number    AS block_number,
       b.timestamp AS block_time,
       i.instruction_id as instruction_id,
       m.to        AS to_derive_address,
       a.owner    AS to_owner_address,
       m.amount
FROM spl2.mints m
         INNER JOIN instructions i ON i.instruction_id = m.instruction_id
         INNER JOIN _blocks_ b ON b.number = i.block_number
         LEFT JOIN initialized_accounts a ON a.account = m."to";

insert into mv_all_mints
SELECT b.number    AS block_number,
       b.timestamp AS block_time,
       i.instruction_id as instruction_id,
       m.to        AS to_derive_address,
       a.owner    AS to_owner_address,
       m.amount
FROM spl2.mints m
         INNER JOIN spl2.instructions i ON i.instruction_id = m.instruction_id
         INNER JOIN spl2._blocks_ b ON b.number = i.block_number
         LEFT JOIN spl2.initialized_accounts a ON a.account = m."to";


-- -----------------------------------------------
-- -----------------------------------------------
-- mv_all_burns
-- -----------------------------------------------
-- -----------------------------------------------
CREATE MATERIALIZED VIEW mv_all_burns
    ENGINE = ReplacingMergeTree()
    ORDER BY (instruction_id)
AS
SELECT b.number    AS block_number,
       b.timestamp AS block_time,
       i.instruction_id AS instruction_id,
       br.from     AS from_derive_address,
       a.owner    AS from_owner_address,
       br.amount
FROM spl2.burns br
         INNER JOIN spl2.instructions i ON i.instruction_id = br.instruction_id
         INNER JOIN spl2._blocks_ b ON b.number = i.block_number
         LEFT JOIN spl2.initialized_accounts a ON a.account = br."from";

insert into mv_all_burns
SELECT b.number    AS block_number,
       b.timestamp AS block_time,
       i.instruction_id AS instruction_id,
       br.from     AS from_derive_address,
       a.owner    AS from_owner_address,
       br.amount
FROM spl2.burns br
         INNER JOIN spl2.instructions i ON i.instruction_id = br.instruction_id
         INNER JOIN spl2._blocks_ b ON b.number = i.block_number
         LEFT JOIN spl2.initialized_accounts a ON a.account = br."from";
-- -----------------------------------------------
-- -----------------------------------------------
-- -----------------------------------------------
-- -----------------------------------------------
CREATE MATERIALIZED VIEW mv_mint_per_month
    ENGINE = ReplacingMergeTree()
    PARTITION BY toYYYYMM(month)
    ORDER BY (month)
AS
SELECT DATE_TRUNC('month', m.block_time) as month,
       sum(m.amount)                     as total
FROM spl2.mv_all_mints m
GROUP BY DATE_TRUNC('month', m.block_time);

insert into mv_mint_per_month
SELECT DATE_TRUNC('month', m.block_time) as month,
       sum(m.amount)                     as total
FROM spl2.mv_all_mints m
GROUP BY DATE_TRUNC('month', m.block_time);


-- -----------------------------------------------
-- -----------------------------------------------
-- Burn per Month --
-- -----------------------------------------------
-- -----------------------------------------------
CREATE MATERIALIZED VIEW mv_burn_per_month
    ENGINE = ReplacingMergeTree()
    PARTITION BY toYYYYMM(month)
    ORDER BY (month)
AS
SELECT DATE_TRUNC('month', m.block_time) as month,
       sum(m.amount)                     as total
FROM spl2.mv_all_burns m
GROUP BY DATE_TRUNC('month', m.block_time);

insert into mv_burn_per_month
SELECT DATE_TRUNC('month', m.block_time) as month,
       sum(m.amount)                     as total
FROM spl2.mv_all_burns m
GROUP BY DATE_TRUNC('month', m.block_time);

-- -----------------------------------------------
-- -----------------------------------------------
-- Total Supply
-- -----------------------------------------------
-- -----------------------------------------------
CREATE MATERIALIZED VIEW mv_supply
    ENGINE = ReplacingMergeTree()
    ORDER BY (stat)
AS
select 'total_supply' as stat,
       ((select sum(total) from spl2.mv_mint_per_month) -
        (select sum(total) from spl2.mv_burn_per_month)) as total_supply;

insert into mv_supply
select 'total_supply' as stat,
       ((select sum(total) from spl2.mv_mint_per_month) -
        (select sum(total) from spl2.mv_burn_per_month)) as total_supply;


SET use_query_cache = 0;
select (select max(number) from _blocks_)        block_max,
       (select max(block_number) from mints)        mints_block_max,
       (select max(block_number) from mv_all_mints) all_mint_block_max,
       (select count() from mints)        as        mints,
       (select count() from mv_all_mints) as        all_mints,
       (select max(block_number) from burns)  burns_block_max,
       (select max(block_number) from mv_all_burns) all_burns_block_max,
       (select count() from burns)        as burns,
       (select count() from mv_all_burns) as all_burns;


SELECT
    block_number,
    instruction_id,
    COUNT(*) AS count
FROM
    spl2.mv_all_mints
GROUP BY
    block_number, instruction_id

HAVING
    count > 1
ORDER BY
    count DESC;

SELECT
    instruction_id,
    COUNT(*) AS count
FROM
    spl2.instructions
GROUP BY
    instruction_id
HAVING
    count > 1
ORDER BY
    count DESC;

SELECT
    number,
    COUNT(*) AS count
FROM
    spl2._blocks_
GROUP BY
    number
HAVING
    count > 1
ORDER BY
    count DESC;



insert into spl2._blocks_ SELECT * FROM spl2._blocks_old;



CREATE TABLE IF NOT EXISTS spl2._blocks_  (
                                              number    integer,
                                              hash      text,
                                              timestamp timestamp
)
    ENGINE = ReplacingMergeTree()
    PRIMARY KEY (number)