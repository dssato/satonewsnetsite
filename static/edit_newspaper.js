let urlPicker = document.getElementById("paper-url");
if (urlPicker) urlPicker.addEventListener("change", (e) => {
    e.target.value = e.target.value.toLowerCase().replace(/[^A-Za-z]/gmi, '-');
});

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
            paper: {
                id: chk(submissionId()),
                name: chk(document.getElementById("paper-name").value),
                featured_issue: parseInt(chk(document.getElementById("paper-featured-issue").value)),
                logo: chk(document.getElementById("paper-logo").value)
            },
            code: parseInt(chk(document.getElementById("veri-code").value)),
        };

        let req = new XMLHttpRequest();
        req.open("POST", "/api/create_paper", true);
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

        pubStatus.innerHTML = "Publishing..."
        req.send(JSON.stringify(data));
    } catch (er) {
        pubStatus.innerHTML = "Some fields have not been filled out!"
    }
}
