///PySql: gen select*,update*,insert*,delete* ... methods
#[macro_export]
macro_rules! crud {
    ($table:ty{}) => {
        $crate::impl_insert!($table {});
        $crate::impl_select!($table {});
        $crate::impl_update!($table {});
        $crate::impl_delete!($table {});
    };
    ($table:ty{},$table_name:expr) => {
        $crate::impl_insert!($table {}, $table_name);
        $crate::impl_select!($table {}, $table_name);
        $crate::impl_update!($table {}, $table_name);
        $crate::impl_delete!($table {}, $table_name);
    };
}

///PySql: gen sql => INSERT INTO table_name (column1,column2,column3,...) VALUES (value1,value2,value3,...);
///
/// example:
/// ```rust
/// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// pub struct BizActivity{ pub id: Option<String> }
/// rbatis::impl_insert!(BizActivity{});
/// ```
///
#[macro_export]
macro_rules! impl_insert {
    ($table:ty{}) => {
        $crate::impl_insert!(
            $table {},
            $crate::utils::string_util::to_snake_name(stringify!($table))
        );
    };
    ($table:ty{},$table_name:expr) => {
        impl $table {
            pub fn insert_batch(
                // executor: &dyn $crate::executor::Executor,
                tables: &[$table],
                batch_size: u64,
            ) -> std::result::Result<Vec<(String, Vec<rbs::Value>, bool)>, $crate::rbdc::Error> {
                #[$crate::py_sql(
                    "`insert into ${table_name} `
                    trim ',':
                     for idx,table in tables:
                      if idx == 0:
                         `(`
                         trim ',':
                           for k,v in table:
                              if k == 'id' && v== null:
                                 continue:
                              ${k},
                         `) VALUES `
                      (
                      trim ',':
                       for k,v in table:
                         if k == 'id' && v== null:
                            continue:
                         #{v},
                      ),
                    "
                )]
                fn insert_batch(
                    // executor: &dyn $crate::executor::Executor,
                    tables: &[$table],
                    table_name: &str,
                ) -> std::result::Result<(String, Vec<rbs::Value>), $crate::rbdc::Error> {
                    impled!()
                }
                if tables.is_empty() {
                    return Err($crate::rbdc::Error::from(
                        "insert can not insert empty array tables!",
                    ));
                }
                let table_name = $table_name.to_string();
                // let mut result = $crate::rbdc::db::ExecResult {
                //     rows_affected: 0,
                //     last_insert_id: rbs::Value::Null,
                // };
                let mut res = Vec::<(String, Vec<rbs::Value>, bool)>::new();
                let ranges = $crate::sql::Page::<()>::make_ranges(tables.len() as u64, batch_size);
                for (offset, limit) in ranges {
                    let result = insert_batch(
                        // executor,
                        &tables[offset as usize..limit as usize],
                        table_name.as_str(),
                    )?;
                    res.push((result.0, result.1, false));
                    // .await?;
                    // result.rows_affected += exec_result.rows_affected;
                    // result.last_insert_id = exec_result.last_insert_id;
                }
                Ok(res)
            }

            pub fn insert(
                table: &$table,
            ) -> std::result::Result<Vec<(String, Vec<rbs::Value>, bool)>, $crate::rbdc::Error> {
                <$table>::insert_batch(&[table.clone()], 1)
            }
        }
    };
}

///PySql: gen sql => SELECT (column1,column2,column3,...) FROM table_name (column1,column2,column3,...)  *** WHERE ***
///
/// example:
///```rust
/// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// pub struct BizActivity{ pub id: Option<String> }
///rbatis::impl_select!(BizActivity{});
///rbatis::impl_select!(BizActivity{select_all_by_id(id:&str,name:&str) => "where id = #{id} and name = #{name}"});
///rbatis::impl_select!(BizActivity{select_by_id(id:String) -> Option => "where id = #{id} limit 1"});
///
/// //use
/// //BizActivity::select**()
/// ```
///
#[macro_export]
macro_rules! impl_select {
    ($table:ty{}) => {
        $crate::impl_select!($table{},$crate::utils::string_util::to_snake_name(stringify!($table)));
    };
    ($table:ty{},$table_name:expr) => {
        $crate::impl_select!($table{select_all() => ""},$table_name);
        $crate::impl_select!($table{select_by_column<V:serde::Serialize>(column: &str,column_value: V) -> Vec => "` where ${column} = #{column_value}`"},$table_name);
        $crate::impl_select!($table{select_in_column<V:serde::Serialize>(column: &str,column_values: &[V]) -> Vec =>
         "` where ${column} in (`
          trim ',': for _,item in column_values:
             #{item},
          `)`"},$table_name);
    };
    ($table:ty{$fn_name:ident $(< $($gkey:ident:$gtype:path $(,)?)* >)? ($($param_key:ident:$param_type:ty $(,)?)*) => $sql:expr}$(,$table_name:expr)?) => {
        $crate::impl_select!($table{$fn_name$(<$($gkey:$gtype,)*>)?($($param_key:$param_type,)*) ->Vec => $sql}$(,$table_name)?);
    };
    ($table:ty{$fn_name:ident $(< $($gkey:ident:$gtype:path $(,)?)* >)? ($($param_key:ident:$param_type:ty $(,)?)*) -> $container:tt => $sql:expr}$(,$table_name:expr)?) => {
        impl $table{
            pub fn $fn_name $(<$($gkey:$gtype,)*>)? ($($param_key:$param_type,)*) -> std::result::Result<(String, Vec<rbs::Value>, bool),$crate::rbdc::Error>
            {
                     #[$crate::py_sql("`select `
                        trim ',':
                            for k,_ in table:
                                ${k}, 
                        ` from ${table_name} `",$sql)]
                     fn $fn_name$(<$($gkey: $gtype,)*>)?(table: &$table,table_name:&str,$($param_key:$param_type,)*) -> std::result::Result<(String, Vec<rbs::Value>),$crate::rbdc::Error> {impled!()}

                     let default_table: $table = Default::default();
                     let mut table_name = String::new();
                     $(table_name = $table_name.to_string();)?
                     if table_name.is_empty(){
                         table_name = $crate::utils::string_util::to_snake_name(stringify!($table));
                     }
                     let result = $fn_name(&default_table,&table_name,$($param_key ,)*)?;
                     Ok((result.0, result.1, false))
            }
        }
    };
}

/// PySql: gen sql = UPDATE table_name SET column1=value1,column2=value2,... WHERE some_column=some_value;
/// ```rust
/// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// pub struct BizActivity{ pub id: Option<String> }
/// rbatis::impl_update!(BizActivity{});
/// ```
#[macro_export]
macro_rules! impl_update {
    ($table:ty{}) => {
        $crate::impl_update!(
            $table{},
            $crate::utils::string_util::to_snake_name(stringify!($table))
        );
    };
    ($table:ty{},$table_name:expr) => {
        $crate::impl_update!($table{update_by_column_value(column: &str,column_value: &rbs::Value) => "`where ${column} = #{column_value}`"},$table_name);
        impl $table {
            pub fn update_by_column(
                table: &$table,
                column: &str) -> std::result::Result<(String, Vec<rbs::Value>, bool), $crate::rbdc::Error>{
                let columns = rbs::to_value!(table);
                let column_value = &columns[column];
                <$table>::update_by_column_value(table,column,column_value)
            }

            pub fn update_by_column_batch(
                tables: &[$table],
                column: &str,
                batch_size: u64,
            ) -> std::result::Result<Vec<(String, Vec<rbs::Value>, bool)>, $crate::rbdc::Error> {
                let ranges = $crate::sql::Page::<()>::make_ranges(tables.len() as u64, batch_size);
                let mut res = Vec::<(String, Vec<rbs::Value>, bool)>::new();
                for (offset, limit) in ranges {
                    //todo better way impl batch?
                    for table in &tables[offset as usize..limit as usize]{
                       let result = <$table>::update_by_column(table,column)?;
                       res.push((result.0, result.1, false));
                    }
                }
                Ok(res)
            }
        }
    };
    ($table:ty{$fn_name:ident($($param_key:ident:$param_type:ty$(,)?)*) => $sql_where:expr}$(,$table_name:expr)?) => {
        impl $table {
            pub fn $fn_name(
                table: &$table,
                $($param_key:$param_type,)*
            ) -> std::result::Result<(String, Vec<rbs::Value>, bool), $crate::rbdc::Error> {
                if $sql_where.is_empty(){
                    return Err($crate::rbdc::Error::from("sql_where can't be empty!"));
                }
                #[$crate::py_sql("`update ${table_name} set `
                                 trim ',':
                                   for k,v in table:
                                     if k == column || v== null:
                                        continue:
                                     `${k}=#{v},`
                                 ` `",$sql_where)]
                  fn $fn_name(
                      table_name: String,
                      table: &rbs::Value,
                      $($param_key:$param_type,)*
                  ) -> std::result::Result<(String, Vec<rbs::Value>), $crate::rbdc::Error> {
                      impled!()
                  }
                  let mut table_name = String::new();
                  $(table_name = $table_name.to_string();)?
                  if table_name.is_empty(){
                      table_name = $crate::utils::string_util::to_snake_name(stringify!($table));
                  }
                  let table = rbs::to_value!(table);
                  let result = $fn_name(table_name, &table, $($param_key,)*)?;
                  Ok((result.0, result.1, false))
            }
        }
    };
}

/// PySql: gen sql = DELETE FROM table_name WHERE some_column=some_value;
///
/// ```rust
/// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// pub struct BizActivity{}
/// rbatis::impl_delete!(BizActivity{});
/// ```
#[macro_export]
macro_rules! impl_delete {
    ($table:ty{}) => {
        $crate::impl_delete!(
            $table{},
            $crate::utils::string_util::to_snake_name(stringify!($table))
        );
    };
    ($table:ty{},$table_name:expr) => {
        $crate::impl_delete!($table {delete_by_column<V:serde::Serialize>(column:&str,column_value: V) => "`where ${column} = #{column_value}`"},$table_name);
        $crate::impl_delete!($table {delete_in_column<V:serde::Serialize>(column:&str,column_values: &[V]) =>
        "`where ${column} in (`
          trim ',': for _,item in column_values:
             #{item},
          `)`"},$table_name);

        impl $table {
            pub fn delete_by_column_batch<V:serde::Serialize>(
                column: &str,
                values: &[V],
                batch_size: u64,
            ) -> std::result::Result<Vec<(String, Vec<rbs::Value> ,bool)>, $crate::rbdc::Error> {
                let mut res = Vec::<(String, Vec<rbs::Value>, bool)>::new();
                let ranges = $crate::sql::Page::<()>::make_ranges(values.len() as u64, batch_size);
                for (offset, limit) in ranges {
                    let result = <$table>::delete_in_column(column,&values[offset as usize..limit as usize])?;
                    res.push((result.0, result.1, false));
                }
                Ok(res)
            }
        }
    };
    ($table:ty{$fn_name:ident $(< $($gkey:ident:$gtype:path $(,)?)* >)? ($($param_key:ident:$param_type:ty$(,)?)*) => $sql_where:expr}$(,$table_name:expr)?) => {
        impl $table {
            pub fn $fn_name$(<$($gkey:$gtype,)*>)?(
                $($param_key:$param_type,)*
            ) -> std::result::Result<(String, Vec<rbs::Value>, bool), $crate::rbdc::Error> {
                if $sql_where.is_empty(){
                    return Err($crate::rbdc::Error::from("sql_where can't be empty!"));
                }
                #[$crate::py_sql("`delete from ${table_name} `",$sql_where)]
                fn $fn_name$(<$($gkey: $gtype,)*>)?(
                    table_name: String,
                    $($param_key:$param_type,)*
                ) -> std::result::Result<(String, Vec<rbs::Value>), $crate::rbdc::Error> {
                    impled!()
                }
                let mut table_name = String::new();
                $(table_name = $table_name.to_string();)?
                if table_name.is_empty(){
                  table_name = $crate::utils::string_util::to_snake_name(stringify!($table));
                }
                let sql = $fn_name(table_name, $($param_key,)*)?;
                Ok((sql.0, sql.1, false))
            }
        }
    };
}

/// pysql impl_select_page
///
/// do_count: default do_count is a bool param value to determine the statement type
///
/// ```rust
/// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// pub struct BizActivity{}
/// rbatis::impl_select_page!(BizActivity{select_page() =>"
///      if do_count == false:
///        `order by create_time desc`"});
/// ```
///
/// limit_sql: If the database does not support the statement `limit ${page_no},${page_size}`,You should add param 'limit_sql:&str'
/// ```rust
/// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// pub struct BizActivity{}
/// rbatis::impl_select_page!(BizActivity{select_page(limit_sql:&str) =>"
///      if do_count == false:
///        `order by create_time desc`"});
/// ```
/// you can see ${page_no} = (page_no -1) * page_size;
/// you can see ${page_size} = page_size;
#[macro_export]
macro_rules! impl_select_page {
    ($table:ty{$fn_name:ident($($param_key:ident:$param_type:ty$(,)?)*) => $where_sql:expr}) => {
        $crate::impl_select_page!(
            $table{$fn_name($($param_key:$param_type)*)=> $where_sql},
            $crate::utils::string_util::to_snake_name(stringify!($table))
        );
    };
    ($table:ty{$fn_name:ident($($param_key:ident:$param_type:ty$(,)?)*) => $where_sql:expr},$table_name:expr) => {
        impl $table {
            pub fn $fn_name(
                page_request: &dyn $crate::sql::IPageRequest,
                $($param_key:$param_type,)*
            ) -> std::result::Result<Vec<(String, Vec<rbs::Value>, bool)>, $crate::rbdc::Error> {
                let default_table: $table = Default::default();
                let mut table_name = $table_name.to_string();
                //pg,mssql can override this parameter to implement its own limit statement
                let mut limit_sql = " limit ${page_no},${page_size}".to_string();
                limit_sql=limit_sql.replace("${page_no}", &page_request.offset().to_string());
                limit_sql=limit_sql.replace("${page_size}", &page_request.page_size().to_string());
                let records:Vec<$table>;
                struct Inner{}
                impl Inner{
                 #[$crate::py_sql(
                    "`select `
                    if do_count == false:
                        trim ',':
                            for k,_ in table:
                                ${k}, 
                    if do_count == true:
                       `count(1) as count`
                    ` from ${table_name} `\n",$where_sql,"\n
                    if do_count == false:
                        `${limit_sql}`")]
                   fn $fn_name(do_count:bool,table:&$table,table_name: &str,page_no:u64,page_size:u64,page_offset:u64,limit_sql:&str,$($param_key:&$param_type,)*) -> std::result::Result<(String, Vec<rbs::Value>), $crate::rbdc::Error> {impled!()}
                }
                let mut res = Vec::<(String, Vec<rbs::Value>, bool)>::new();
                let mut total = 0;
                if page_request.do_count() {
                    let total_value = Inner::$fn_name(true,&default_table,&table_name,page_request.page_no(), page_request.page_size(),page_request.offset(),"",$(&$param_key,)*)?;
                    res.push((total_value.0, total_value.1, true));
                }
                let mut page = $crate::sql::Page::<$table>::new_total(page_request.page_no(), page_request.page_size(), total);
                let records_value = Inner::$fn_name(false,&default_table,&table_name,page_request.page_no(), page_request.page_size(),page_request.offset(),&limit_sql,$(&$param_key,)*)?;
                res.push((records_value.0, records_value.1, false));
                Ok(res)
            }
        }
    };
}

/// impl html_sql select page.
///
/// you must deal with 3 param:
/// (do_count:bool,page_no:u64,page_size:u64)
///
/// you must deal with sql:
/// return Vec<Record>（if param do_count = false）
/// return u64（if param do_count = true）
///
/// you can see ${page_no} = (page_no -1) * page_size;
/// you can see ${page_size} = page_size;
///
/// just like this example:
/// ```html
/// <select id="select_page_data">
///         `select `
///         <if test="do_count == true">
///             `count(1) from table`
///         </if>
///         <if test="do_count == false">
///             `* from table limit ${page_no},${page_size}`
///         </if>
///   </select>
/// ```
/// ```
/// #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
/// pub struct BizActivity{}
/// //rbatis::htmlsql_select_page!(select_page_data(name: &str) -> BizActivity => "example.html");
/// rbatis::htmlsql_select_page!(select_page_data(name: &str) -> BizActivity => r#"<select id="select_page_data">`select `<if test="do_count == true">`count(1) from table`</if><if test="do_count == false">`* from table limit ${page_no},${page_size}`</if></select>"#);
/// ```
#[macro_export]
macro_rules! htmlsql_select_page {
    ($fn_name:ident($($param_key:ident:$param_type:ty$(,)?)*) -> $table:ty => $html_file:expr) => {
            pub fn $fn_name(page_request: &dyn $crate::sql::IPageRequest, $($param_key:$param_type,)*) -> std::result::Result<Vec<(std::string::String, Vec<rbs::Value>, bool)>, $crate::rbdc::Error> {
            struct Inner{}
            impl Inner{
              #[$crate::html_sql($html_file)]
              pub fn $fn_name(do_count:bool,page_no:u64,page_size:u64,$($param_key: &$param_type,)*) -> std::result::Result<(std::string::String, Vec<rbs::Value>), $crate::rbdc::Error>{
                 $crate::impled!()
              }
            }
            let mut res = Vec::<(String, Vec<rbs::Value>, bool)>::new();
            let mut total = 0;
            if page_request.do_count() {
               let total_value = Inner::$fn_name(true, page_request.offset(), page_request.page_size(), $(&$param_key,)*)?;
               res.push((total_value.0, total_value.1, true));
            }
            let mut page = $crate::sql::Page::<$table>::new_total(page_request.page_no(), page_request.page_size(), total);
            let records_value = Inner::$fn_name(false, page_request.offset(), page_request.page_size(), $(&$param_key,)*)?;
            res.push((records_value.0, records_value.1, false));
            Ok(res)
         }
    }
}

/// impl py_sql select page.
///
/// you must deal with 3 param:
/// (do_count:bool,page_no:u64,page_size:u64)
///
/// you must deal with sql:
/// return Vec<Record>（if param do_count = false）
/// return u64（if param do_count = true）·
///
/// you can see ${page_no} = (page_no -1) * page_size;
/// you can see ${page_size} = page_size;
///
/// just like this example:
/// ```py
/// `select * from activity where delete_flag = 0`
///                   if name != '':
///                     ` and name=#{name}`
///                   if !ids.is_empty():
///                     ` and id in `
///                     ${ids.sql()}
/// ```
/// ```
/// #[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
/// pub struct MockTable{}
/// rbatis::pysql_select_page!(pysql_select_page(name:&str) -> MockTable =>
///     r#"`select `
///       if do_count == true:
///         ` count(1) as count `
///       if do_count == false:
///          ` * `
///       `from activity where delete_flag = 0`
///         if name != '':
///            ` and name=#{name}`
///       ` limit ${page_no},${page_size}`
/// "#);
/// ```
#[macro_export]
macro_rules! pysql_select_page {
    ($fn_name:ident($($param_key:ident:$param_type:ty$(,)?)*) -> $table:ty => $py_file:expr) => {
            pub fn $fn_name(page_request: &dyn $crate::sql::IPageRequest, $($param_key:$param_type)*) -> std::result::Result<Vec<(std::string::String, Vec<rbs::Value>, bool)>, $crate::rbdc::Error> {
            struct Inner{}
            impl Inner{
              #[$crate::py_sql($py_file)]
              pub fn $fn_name(do_count:bool,page_no:u64,page_size:u64,$($param_key: &$param_type)*) -> std::result::Result<(std::string::String, Vec<rbs::Value>), $crate::rbdc::Error>{
                 $crate::impled!()
              }
            }
            let mut res = Vec::<(String, Vec<rbs::Value>, bool)>::new();
            let mut total = 0;
            if page_request.do_count() {
               let total_value = Inner::$fn_name(true, page_request.offset(), page_request.page_size(), $(&$param_key)*)?;
               res.push((total_value.0, total_value.1, true));
            }
            let mut page = $crate::sql::Page::<$table>::new_total(page_request.page_no(), page_request.page_size(), total);
            let records_value = Inner::$fn_name(false, page_request.offset(), page_request.page_size(), $(&$param_key)*)?;
            res.push((records_value.0, records_value.1, false));
            Ok(res)
         }
    }
}

/// use macro wrapper #[py_sql]
/// for example:
/// ```rust
/// use rbatis::executor::Executor;
/// rbatis::pysql!(test_same_id(id: &u64)  -> Result<(std::string::String, Vec<rbs::Value>), rbatis::Error> =>
/// "select * from table where ${id} = 1
///  if id != 0:
///    `id = #{id}`"
/// );
/// ```
#[macro_export]
macro_rules! pysql {
    ($fn_name:ident($($param_key:ident:$param_type:ty$(,)?)*) -> $return_type:ty => $py_file:expr) => {
       pub fn $fn_name($($param_key: $param_type,)*) -> $return_type{
           pub struct Inner{};
           impl Inner{
               #[$crate::py_sql($py_file)]
               pub fn $fn_name($($param_key: $param_type,)*) -> $return_type{
                 impled!()
               }
           }
           Inner::$fn_name($($param_key,)*)
       }
    }
}

/// use macro wrapper #[html_sql]
/// for example:
/// ```rust
/// use rbatis::executor::Executor;
/// rbatis::htmlsql!(test_same_id(id: &u64)  -> Result<(std::string::String, Vec<rbs::Value>), rbatis::Error> => r#"<mapper>
///             <select id="test_same_id">
///             select ${id},${id},#{id},#{id}
///             </select>
///             </mapper>"#);
/// ```
/// or load from file
/// ```rust
/// //use rbatis::executor::Executor;
/// //rbatis::htmlsql!(test_same_id(rb: &dyn Executor, id: &u64)  -> Result<rbs::Value, rbatis::Error> => "example.html");
/// ```
#[macro_export]
macro_rules! htmlsql {
    ($fn_name:ident($($param_key:ident:$param_type:ty$(,)?)*) -> $return_type:ty => $html_file:expr) => {
        pub fn $fn_name($($param_key: $param_type,)*) -> $return_type{
            pub struct Inner{};
            impl Inner{
            #[$crate::html_sql($html_file)]
            pub fn $fn_name($($param_key: $param_type,)*) -> $return_type{
              impled!()
             }
           }
           Inner::$fn_name($($param_key,)*)
        }
    }
}
