


export class Graphics {
    constructor(canvas_id, name) {
        this.canvas = document.getElementById(canvas_id)
        this.canvas.width = document.body.clientWidth; //document.width is obsolete
        this.canvas.height = document.body.clientHeight; //document.height is obsolete
        this.canvas_width = this.canvas.width;
        this.canvas_height = this.canvas.height;
        this.ctx = this.canvas.getContext("2d");
        this.connection = new WebSocket("ws://" + location.host + "/ws/")

        this.game_state = {
            x: 20,
            y: 20,
        }

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
            this.game_state = JSON.parse(e.data);
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

        document.addEventListener("keydown", keyHandlerFactory(true), false);
        document.addEventListener("keyup", keyHandlerFactory(false), false);
    }

    flip_y(y) {
        return this.canvas.height - y;
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
                    ...k
                }
            );
            if (this.connection.readyState === WebSocket.OPEN) {
                this.connection.send(s);
            }

            ctx.beginPath();
            ctx.arc(this.game_state.x, this.flip_y(this.game_state.y), 20, 0, 2 * Math.PI);
            ctx.stroke();

            requestAnimationFrame(loop);
        }

        requestAnimationFrame(loop);
    }
}
