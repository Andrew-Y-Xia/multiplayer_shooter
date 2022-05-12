

export class Graphics {
    constructor(canvas_id) {
        this.canvas = document.getElementById("canvas1")
        this.ctx = this.canvas.getContext("2d");
        this.connection = new WebSocket("ws://" + location.host + "/ws/")

        // When the connection is open, send some data to the server
        this.connection.onopen = function () {
        };

        // Log errors
        this.connection.onerror = function (error) {
            console.log('WebSocket Error ' + error);
        };

        // Log messages from the server
        this.connection.onmessage = function (e) {
            console.log('Server: ' + e.data);
        };
    }

    run() {
        const ctx = this.ctx;
        const canvas = this.canvas;
        function loop() {
            ctx.clearRect(0, 0, canvas.width, canvas.height);

            

            ctx.beginPath();
            ctx.arc(95, 50, 20, 0, 2 * Math.PI);
            ctx.stroke();

            requestAnimationFrame(loop);
        }

        requestAnimationFrame(loop);
    }
}