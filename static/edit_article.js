let urlPicker = document.getElementById("article-url");
if (urlPicker) urlPicker.addEventListener("change", (e) => {
    e.target.value = e.target.value.toLowerCase().replace(/[^A-Za-z]/gmi, '-');
});

var article = new Quill("#article-body", {
    theme: "snow",
    modules: {
        toolbar: [
            [{ header: [1, 2, 3, 4, false] }],
            ["bold", "italic", "underline", "strike"],
            ["link", "video"],
            [{ list: "ordered"}, { list: "bullet" }],
            [{ script: "sub"}, { script: "super" }],
            [{ color: [] }, { background: [] }],
            [{ align: [] }],
            ["clean"]
        ]
    }
});

const months = ["January", "February", "March", "April", "May", "June", "July", "August", "September", "October", "November", "December"];

function chk(v) {
    if (v === undefined || v == "") {
        throw new Error();
    }
    return v;
}

function onSubmit() {
    let pubStatus = document.getElementById("pub-status");

    try {
        let data = {
            article: {
                id: chk(submissionId()),
                title: chk(document.getElementById("article-title").value),
                author: chk(document.getElementById("article-author").value),
                date: chk(Math.floor(Date.now() / 1000)),
                paper: chk(document.getElementById("article-paper").value),
                issue: chk(parseInt(document.getElementById("article-issue").value)),
                image: chk(document.getElementById("article-image").value),
                style: parseInt(chk(document.getElementById("article-style").value)),
                column: parseInt(chk(document.getElementById("article-column").value)),
                sortnum: parseInt(chk(document.getElementById("article-sortnum").value)),
                content: JSON.stringify(article.getContents()),
            },
            code: parseInt(chk(document.getElementById("veri-code").value)),
        };

        let req = new XMLHttpRequest();
        req.open("POST", "/api/publish", true);
        req.setRequestHeader("Content-Type", "application/json");
        req.onreadystatechange = () => {
            if (req.readyState === 4) {
                if (req.status === 200) {
                    pubStatus.innerHTML = req.responseText;
                } else {
                    pubStatus.innerHTML = `ERROR Publishing! ${req.responseText}`;
                }
            }
        };
        console.log(data);

        pubStatus.innerHTML = "Publishing..."
        req.send(JSON.stringify(data));
    } catch (er) {
        pubStatus.innerHTML = "Some fields have not been filled out!"
    }
}

