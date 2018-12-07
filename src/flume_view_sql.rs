use flume_view::*;

use rusqlite::{Connection, NO_PARAMS};
use serde_json::Value;
use rusqlite::types::{ToSql};
use log;

struct FlumeViewSql {
    connection: Connection,
    latest: Sequence
}

fn create_tables(conn: &mut Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS message (
          id INTEGER PRIMARY KEY,
          key TEXT UNIQUE, 
          seq INTEGER,
          received_time TEXT,
          asserted_time TEXT,
          root TEXT,
          branch TEXT,
          author TEXT,
          content_type TEXT,
          content JSON
        )",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS links (
          id INTEGER PRIMARY KEY,
          links_from INTEGER,
          links_to INTEGER
        )",
        NO_PARAMS,
    )
    .unwrap();

    conn.execute(
        "CREATE TABLE IF NOT EXISTS heads (
          id INTEGER PRIMARY KEY,
          links_from INTEGER,
          links_to INTEGER
        )",
        NO_PARAMS,
    )
    .unwrap();
}

impl FlumeViewSql {
    pub fn new(path: &str, latest: Sequence) -> FlumeViewSql {
        let mut connection = Connection::open(path).expect("unable to open sqlite connection");

        create_tables(&mut connection);

        FlumeViewSql { connection, latest }
    }
}

impl FlumeView for FlumeViewSql {
    fn append(&mut self, seq: Sequence, item: &[u8]){
        // ToSql is not implemented for u64. I think it's pretty safe to cast u64 to i64. The file
        // would have to be very very large (2^63) before it overflows.
        let signed_seq = seq as i64;

        serde_json::from_slice(item)
            .map(|message: SsbMessage|{
                self.connection.execute(
                    "INSERT INTO message (id, key, seq, received_time, asserted_time, root, branch, author, content_type, content)
                  VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                  &[
                    &signed_seq as &ToSql, 
                    &message.key, 
                    &message.value.sequence, 
                    &message.timestamp, 
                    &message.value.timestamp, 
                    &message.value.content["root"] as &ToSql,
                    &message.value.content["branch"] as &ToSql ,
                    &message.value.author,
                    &message.value.content["type"] as &ToSql,
                    &message.value.content as &ToSql ,
                  ],
                  ).unwrap();
            
            })
            .unwrap_or({
                warn!("Unable to parse item as a ssb message, seq: {}", seq);
            });
    }
    fn latest(&self) -> Sequence{
        self.latest
    }
}

#[derive(Serialize, Deserialize)]
struct SsbValue {
    author: String,
    sequence: u32,
    timestamp: i64,
    content: Value,
}

#[derive(Serialize, Deserialize)]
struct SsbMessage {
    key: String,
    value: SsbValue,
    timestamp: i64 
}


#[cfg(test)]
mod test {
    use flume_view_sql::*;
    use flume_view::*;
    use serde_json::*;

    #[test]
    fn open_connection(){
        FlumeViewSql::new("/tmp/test.sqlite3", 0);
        assert!(true)
    }

    #[test]
    fn append(){
        let filename = "/tmp/test.sqlite3";
        std::fs::remove_file(filename.clone())
            .or::<Result<()>>(Ok(()))
            .unwrap();

        let mut view = FlumeViewSql::new(filename, 0);
        let jsn = r#####"{
  "key": "%KKPLj1tWfuVhCvgJz2hG/nIsVzmBRzUJaqHv+sb+n1c=.sha256",
  "value": {
    "previous": "%xsMQA2GrsZew0GSxmDSBaoxDafVaUJ07YVaDGcp65a4=.sha256",
    "author": "@QlCTpvY7p9ty2yOFrv1WU1AE88aoQc4Y7wYal7PFc+w=.ed25519",
    "sequence": 4797,
    "timestamp": 1543958997985,
    "hash": "sha256",
    "content": {
      "type": "post",
      "root": "%9EdpeKC5CgzpQs/x99CcnbD3n6ugUlwm19F7ZTqMh5w=.sha256",
      "branch": "%sQV8QpyUNvh7fBAs2ts00Qo2gj44CQBmwonWJzm+AeM=.sha256",
      "reply": {
        "%9EdpeKC5CgzpQs/x99CcnbD3n6ugUlwm19F7ZTqMh5w=.sha256": "@+UMKhpbzXAII+2/7ZlsgkJwIsxdfeFi36Z5Rk1gCfY0=.ed25519",
        "%sQV8QpyUNvh7fBAs2ts00Qo2gj44CQBmwonWJzm+AeM=.sha256": "@vzoU7/XuBB5B0xueC9NHFr9Q76VvPktD9GUkYgN9lAc=.ed25519"
      },
      "channel": null,
      "recps": null,
      "text": "If I understand correctly, cjdns overlaying over old IP (which is basically all of the cjdns uses so far) still requires old IP addresses to introduce you to the cjdns network, so the chicken and egg problem is still there.",
      "mentions": []
    },
    "signature": "mi5j/buYZdsiH8l6CVWRqdBKe+0UG6tVTOoVVjMhYl38Nkmb8wiIEfe7zu0JWuiHkaAIq+0/ZqYr6aV14j4fAw==.sig.ed25519"
  },
  "timestamp": 1543959001933
}
"#####;
        view.append(0, jsn.as_bytes());
    }
}
