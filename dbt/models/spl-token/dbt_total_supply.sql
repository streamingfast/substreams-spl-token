{{ config(materialized='table') }}

select (SELECT SUM(amount) FROM spl.mints) - (SELECT COALESCE(SUM(amount), 0)  FROM spl.burns) as total_supply