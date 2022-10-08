const maxDescLength = 120;

function getPreviewText(articleJsonStr) {
    let content = JSON.parse(articleJsonStr);
    let out = [];

    for (let op of content.ops) {
        if (op.attributes && (op.attributes.header || op.attributes.insert || op.attributes.list)) {
            out.pop();
        } else if (op.insert && (typeof op.insert === 'string' || op.insert instanceof String)) {
            out.push(op.insert);
        }
    }

    return out.join("").substring(0, maxDescLength) + "...";
}
