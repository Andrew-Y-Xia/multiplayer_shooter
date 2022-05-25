import { Game } from './game.js'


var SETTINGS;

await $.getJSON( "./settings.json", function( data ) {
    SETTINGS = data;
    console.log(SETTINGS);
});


$("#join-button").click(() => {
    const user_name = $("#name-input").val();

    // Entry point for game
    let graphics = new Game("canvas1", user_name);
    setTimeout(() => graphics.run(), 100);
})
