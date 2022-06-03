import {interpolate, render_sprite, translator, render_border, render_background, render_bullet} from './graphics.js';


Array.prototype.pushSorted = function(el, compareFn) {
    this.splice((function(arr) {
      var m = 0;
      var n = arr.length - 1;
  
      while(m <= n) {
        var k = (n + m) >> 1;
        var cmp = compareFn(el, arr[k]);
  
        if(cmp > 0) m = k + 1;
          else if(cmp < 0) n = k - 1;
          else return k;
      }
  
      return -m - 1;
    })(this), 0, el);
  
    return this.length;
  };
  

export class Game {

    constructor(canvas_id, name) {
        this.name = name;
        this.canvas = document.getElementById(canvas_id)
        this.canvas.width = window.innerWidth;
        this.canvas.height = window.innerHeight;
        console.log(this.canvas.width, this.canvas.height);
        this.center = {
            x: this.canvas.width / 2,
            y: this.canvas.height / 2
        };
        this.ctx = this.canvas.getContext("2d");
        this.connection = new WebSocket("ws://" + location.host + "/ws/")


        // Array of game states
        this.game_state_buffer = [];

        

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
            this.insertGameState(data);
            if (this.start_timestamp === undefined) {
                this.start_timestamp = data.timestamp;
                this.js_epoch = performance.now();
            }
            // console.log(this.game_state);
        };

        this.keydown = {
            w: false,
            a: false,
            s: false,
            d: false,
            click: false,
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

        document.body.onmousedown = () => { 
            this.keydown.click = true;
          }
          document.body.onmouseup = () => {
            this.keydown.click = false;
          }

        document.addEventListener("keydown", keyHandlerFactory(true), false);
        document.addEventListener("keyup", keyHandlerFactory(false), false);
        document.addEventListener("mousemove", moveHandler, false);
    }

    // Caculates direction from player sprite to mouse
    getMouseDirs() {
        return Math.atan2(this.center.y - this.mouse_cords.y, this.center.x - this.mouse_cords.x) + Math.PI / 2;
    }

    // Get game state by interpolating between game states in the buffer
    getGameState() {
        let game_state = {
            my_coords : {
                x: 0, y: 0
            },
            enemies : [],
            timestamp: 0,
        };


        let current_rust_timestamp = this.start_timestamp + (performance.now() - this.js_epoch);
        let target_timestamp = current_rust_timestamp - 70;

        /*
        // Alias buffer for less typing
        let buffer = this.game_state_buffer;
        // Do linear search to find two game states to interpolate between
        for (let i = 0; i < buffer.length - 1; i++) {
            if (buffer[i].timestamp >= target_timestamp && buffer[i + 1].timestamp <= target_timestamp) {
                // States found, now linearly interpolate all attributes
                let s1 = buffer[i];
                let s2 = buffer[i + 1];
                let f = (y1, y2) => interpolate(y1, y2, s1.timestamp, s2.timestamp, target_timestamp);

                game_state.my_coords = {
                    x: f(s1.my_coords.x, s2.my_coords.y),
                    y: f(s1.my_coords.y, s2.my_coords.y),
                }
                let 
            }
        }
        */
        
        // Find the closest game_state

        // Alias buffer for less typing
        let buffer = this.game_state_buffer;
        // Do linear search to find two game states target is in between
        for (let i = 0; i < buffer.length - 1; i++) {
            if (buffer[i].timestamp >= target_timestamp && buffer[i + 1].timestamp <= target_timestamp) {
                // States found, now choose the better state
                game_state = Math.abs(buffer[i] - target_timestamp) > Math.abs(buffer[i + 1] - target_timestamp) ? buffer[i] : buffer[i + 1];
                break;
            }
        }

        if (game_state.timestamp === 0) {
            console.log(target_timestamp);
            console.log(buffer);
            game_state = buffer[0];
        }

        return game_state;
    }

    // Insert the game state into buffer and sort
    insertGameState(game_state) {
        this.game_state_buffer.pushSorted(game_state, function(a, b){return a.timestamp - b.timestamp});
        if (this.game_state_buffer.length > 1000) {
            this.game_state_buffer.pop();
        }
    }


    run() {
        const ctx = this.ctx;
        const canvas = this.canvas;
        let loop = () => {

            let original_game_state = this.getGameState();
            let translate = translator(original_game_state.my_coords.x, original_game_state.my_coords.y, this.center.x, this.center.y);
            let t_game_state = {
                my_coords : translate(original_game_state.my_coords.x, original_game_state.my_coords.y),
                enemies: [],
                bullets: []
            }
            for (let i = 0; i < original_game_state.enemies.length; i++) {
                const enemy = original_game_state.enemies[i];
                t_game_state.enemies.push({
                    coords: translate(enemy.coords.x, enemy.coords.y),
                    dir: enemy.dir,
                    username: enemy.username,
                })
            }
            for (let i = 0; i < original_game_state.bullets.length; i++) {
                const bullet = original_game_state.bullets[i];
                t_game_state.bullets.push(translate(bullet.x, bullet.y))
            }
            

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

            let o = translate(0, 0);
            // render_border(ctx, t_origin.x, t_origin.y);
            render_border(ctx, o.x, o.y);
            render_background(ctx, original_game_state.my_coords.x, original_game_state.my_coords.y);

            render_sprite(ctx, t_game_state.my_coords.x, t_game_state.my_coords.y, this.getMouseDirs(), this.name, 'red');
            for (let i = 0; i < t_game_state.enemies.length; i++) {
                const enemy = t_game_state.enemies[i];
                render_sprite(ctx, enemy.coords.x, enemy.coords.y, enemy.dir, enemy.username, 'blue');
            }

            for (let i = 0; i < t_game_state.bullets.length; i++) {
                const bullet = t_game_state.bullets[i];
                render_bullet(ctx, bullet.x, bullet.y);
            }

            requestAnimationFrame(loop);
        }

        requestAnimationFrame(loop);
    }
}
