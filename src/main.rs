mod serializers;
#[cfg(test)]
mod test;

use russenger::prelude::*;
use serializers::load;

#[action]
async fn Main(res: Res, req: Req) {
    res.send(TextModel::new(&req.user, "Hello, world!")).await;
    let _ = load();
}

russenger_app!(Main);
