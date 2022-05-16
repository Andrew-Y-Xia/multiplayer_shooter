
$("#join-button").click(() => {
    sessionStorage.setItem("username", $("#name-input").val());
    window.location.href = "game.html";
})
