pub mod sql {
    pub const GET_STATEMENT_QUERY: &str = r#"
      SELECT c.saldo, c.limite, t.id, t.cliente_id, t.valor as tvalor, t.tipo, t.descricao, t.realizada_em
      FROM public.clientes c
          LEFT JOIN (
              SELECT *
              FROM transacoes
              WHERE cliente_id = $1
              ORDER BY realizada_em DESC
              LIMIT 10
      ) t ON c.id = t.cliente_id
      WHERE c.id = $1;
    "#;
    pub const INSERT_TRANSACTION_QUERY: &str = r#"
      INSERT INTO public.transacoes (cliente_id, valor, tipo, descricao)
      VALUES ($1, $2, $3, $4);
    "#;
    pub const UPDATE_CREDIT_TRANSACTION_QUERY: &str = r#"
      UPDATE public.clientes c
      SET saldo = saldo + $1
      WHERE c.id = $2 RETURNING *;
    "#;
    pub const UPDATE_DEBIT_TRANSACTION_QUERY: &str = r#"
      UPDATE public.clientes c
      SET saldo = saldo - $1
      WHERE c.id = $2 AND saldo - $1 >= - limite RETURNING *;
    "#;
}
