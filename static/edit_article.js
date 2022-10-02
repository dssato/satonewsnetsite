document.getElementById("article-url").addEventListener("change", (e) => {
    e.target.value = e.target.value.toLowerCase().replace(/[^A-Za-z]/gmi, '-');
});

var article = new Quill("#article-body", {
    theme: "snow",
    modules: {
        toolbar: [
            [{ header: [1, 2, 3, 4, false] }],
            ["bold", "italic", "underline", "strike"],
            ["blockquote", "code-block", "link", "image", "video"],
            [{ list: "ordered"}, { list: "bullet" }],
            [{ script: "sub"}, { script: "super" }],
            [{ color: [] }, { background: [] }],
            [{ align: [] }],
            ["clean"]
        ]
    }
});

const months = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

function onSubmit() {
    let data = {
        article: {
            id: document.getElementById("article-url").value,
            title: document.getElementById("article-title").value,
            author: document.getElementById("article-author").value,
            date: Math.floor(Date.now() / 1000),
            paper: document.getElementById("article-paper").value,
            issue: parseInt(document.getElementById("article-issue").value),
            image: document.getElementById("article-image").value,
            content: JSON.stringify(article.getContents()),
        },
        code: parseInt(document.getElementById("veri-code").value),
    };

    let pubStatus = document.getElementById("pub-status");
    let req = new XMLHttpRequest();
    req.open("POST", "/api/publish", true);
    req.setRequestHeader("Content-Type", "application/json");
    req.onreadystatechange = () => {
        if (req.readyState === 4) {
            if (req.status === 200) {
                pubStatus.innerHTML = "Successfully Published!"
            } else {
                pubStatus.innerHTML = `ERROR Publishing! ${req.responseText}`
            }
        }
    };
    console.log(data);

    pubStatus.innerHTML = "Publishing..."
    req.send(JSON.stringify(data));
}

