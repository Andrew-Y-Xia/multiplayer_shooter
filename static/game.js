import { Graphics } from './graphics.js'

let graphics = new Graphics("canvas1", sessionStorage.getItem("username"));
setTimeout(() => graphics.run(), 100);

