{{ config(materialized='table') }}

select (SELECT SUM(amount::NUMERIC) FROM spl.mints) - (SELECT COALESCE(SUM(amount::NUMERIC), 0)  FROM spl.burns) as total_supply