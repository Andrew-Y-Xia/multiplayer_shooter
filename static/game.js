import {render_sprite} from './graphics.js';


export class Game {

    constructor(canvas_id, name) {
        this.name = name;
        this.canvas = document.getElementById(canvas_id)
        this.canvas.width = document.body.clientWidth; //document.width is obsolete
        this.canvas.height = document.body.clientHeight; //document.height is obsolete
        console.log(this.canvas.width, this.canvas.height);
        this.center = {
            x: this.canvas.width / 2,
            y: this.canvas.height / 2
        };
        this.ctx = this.canvas.getContext("2d");
        this.connection = new WebSocket("ws://" + location.host + "/ws/")

        this.game_state = {
            my_coords : {
                x: 0, y: 0
            },
            enemies : [],
        };

        this.mouse_cords = {
            x: 0,
            y: 0,
        };

        // When the connection is open, send some data to the server
        this.connection.onopen = () => {
            this.connection.send(JSON.stringify({type: "JoinGame", username: name}));
        };

        // Log errors
        this.connection.onerror = (error) => {
            console.log('WebSocket Error ' + error);
        };

        // Log messages from the server
        this.connection.onmessage = (e) => {
            
            let data = JSON.parse(e.data);
            this.game_state = data;
            // console.log(this.game_state);
        };

        this.keydown = {
            w: false,
            a: false,
            s: false,
            d: false,
        }

        let keyHandlerFactory = (is_keydown_handler) => {
            let b = is_keydown_handler;
            return (e) => {
                var code = e.keyCode;
                if (code == 87) this.keydown['w'] = b;
                if (code == 65) this.keydown['a'] = b;
                if (code == 83) this.keydown['s'] = b;
                if (code == 68) this.keydown['d'] = b;
            }
        }

        let moveHandler = (e) => {
            this.mouse_cords = {
                x: e.clientX,
                y: e.clientY
            };
        }

        document.addEventListener("keydown", keyHandlerFactory(true), false);
        document.addEventListener("keyup", keyHandlerFactory(false), false);
        document.addEventListener("mousemove", moveHandler, false);
    }

    // Caculates direction from player sprite to mouse
    getMouseDirs() {
        return Math.atan2(this.center.y - this.mouse_cords.y, this.center.x - this.mouse_cords.x) + Math.PI / 2;
    }


    run() {
        const ctx = this.ctx;
        const canvas = this.canvas;
        let loop = () => {
            ctx.clearRect(0, 0, canvas.width, canvas.height);

            
            // Sample events and send back info
            let k = this.keydown;
            let s = JSON.stringify(
                {
                    type: "GameAction", 
                    ...k,
                    dir: this.getMouseDirs(),
                }
            );
            if (this.connection.readyState === WebSocket.OPEN) {
                this.connection.send(s);
            }

            render_sprite(ctx, this.game_state.my_coords.x, this.game_state.my_coords.y, this.getMouseDirs(), this.name, 'red');
            for (let i = 0; i < this.game_state.enemies.length; i++) {
                const enemy = this.game_state.enemies[i];
                render_sprite(ctx, enemy.coords.x, enemy.coords.y, enemy.dir, enemy.username, 'blue');
            }

            requestAnimationFrame(loop);
        }

        requestAnimationFrame(loop);
    }
}
