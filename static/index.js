import { Game } from './game.js';


$("#join-button").click(() => {
    const user_name = $("#name-input").val();

    // Entry point for game
    let game = new Game("canvas1", user_name);
    setTimeout(() => {
        $("#start-panel").hide();
        game.run();
    }, 100);
})
