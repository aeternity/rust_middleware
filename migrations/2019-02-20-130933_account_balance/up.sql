create materialized view if not exists public.account_spendtx_balance as (
select tx->>'recipient_id' as account, SUM(NULLIF(tx->>'amount', '')::decimal) AS amount,
count(hash) as txcount
from transactions t where valid= true and tx_type = 'SpendTx' and tx->>'recipient_id' like 'ak_%'
group by account order by amount desc);

refresh materialized view public.account_spendtx_balance;