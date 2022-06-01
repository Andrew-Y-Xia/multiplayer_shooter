import { Game } from './game.js'


await $.getJSON( "./settings.json", function( data ) {
    let SETTINGS = data;
    console.log(SETTINGS);

    window.game_settings = SETTINGS;
});


$("#join-button").click(() => {
    const user_name = $("#name-input").val();

    // Entry point for game
    let graphics = new Game("canvas1", user_name);
    setTimeout(() => graphics.run(), 100);
})
