var quill = new Quill("#quill-dummy");
quill.setContents(articleContent);

document.getElementById("article").innerHTML = document.querySelector(".ql-editor").innerHTML;
document.getElementById("quill-dummy").remove();
