{{
    config(
        materialized='incremental',
        indexes=[
            {'columns': ['block_number']},
            {'columns': ['block_time']},
            {'columns': ['to_derive_address']},
            {'columns': ['to_owner_address']},
        ]
    )
}}

select
    b.number as block_number,
    b.timestamp as block_time,
    i.transaction_hash,
    m.to as to_derive_address,
    ia.owner as to_owner_address,
    m.amount
from spl.mints m
         inner join spl.instructions i on i.instruction_id = m.instruction_id
         inner join spl.blocks b on b.number = i.block_number
         left join  spl.initialized_accounts ia on ia.account = m."to"
{% if is_incremental() %}
where b.number > (select max(block_number) from {{this}})
{% endif %}