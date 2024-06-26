use crate::structs::{AdminPurgeCommentView, ModlogListParams};
use diesel::{
  result::Error,
  BoolExpressionMethods,
  ExpressionMethods,
  IntoSql,
  JoinOnDsl,
  NullableExpressionMethods,
  QueryDsl,
};
use diesel_async::RunQueryDsl;
use lemmy_db_schema::{
  newtypes::PersonId,
  schema::{admin_purge_comment, person, post},
  utils::{get_conn, limit_and_offset, DbPool},
};

impl AdminPurgeCommentView {
  pub async fn list(pool: &mut DbPool<'_>, params: ModlogListParams) -> Result<Vec<Self>, Error> {
    let conn = &mut get_conn(pool).await?;
    let admin_person_id_join = params.mod_person_id.unwrap_or(PersonId(-1));
    let show_mod_names = !params.hide_modlog_names;
    let show_mod_names_expr = show_mod_names.as_sql::<diesel::sql_types::Bool>();

    let admin_names_join = admin_purge_comment::admin_person_id
      .eq(person::id)
      .and(show_mod_names_expr.or(person::id.eq(admin_person_id_join)));

    let mut query = admin_purge_comment::table
      .left_join(person::table.on(admin_names_join))
      .inner_join(post::table)
      .select((
        admin_purge_comment::all_columns,
        person::all_columns.nullable(),
        post::all_columns,
      ))
      .into_boxed();

    if let Some(admin_person_id) = params.mod_person_id {
      query = query.filter(admin_purge_comment::admin_person_id.eq(admin_person_id));
    };

    // If a post or comment ID is given, then don't find any results
    if params.post_id.is_some() || params.comment_id.is_some() {
      return Ok(vec![]);
    }

    let (limit, offset) = limit_and_offset(params.page, params.limit)?;

    query
      .limit(limit)
      .offset(offset)
      .order_by(admin_purge_comment::when_.desc())
      .load::<AdminPurgeCommentView>(conn)
      .await
  }
}
