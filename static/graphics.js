// ALL FUNCTIONS IN THIS FILE DRAW RELATIVE TO PLAYER
import {get_settings} from './settings.js';


export function render_sprite(ctx, x, y, dir, name, color) {

    ctx.save();
    // Add barrel
    ctx.translate(x, y);
    ctx.rotate(dir);
    ctx.translate(-x, -y);

    const height = 40;
    const width = 20;
    ctx.fillStyle = 'white';
    ctx.fillRect(x - (width / 2), y , width, height);
    ctx.strokeRect(x - (width / 2), y , width, height);
    ctx.restore();
    
    

    // Make the circle first
    ctx.beginPath();
    ctx.arc(x, y, 20, 0, 2 * Math.PI);
    ctx.save()
    ctx.fillStyle = color;
    ctx.fill();
    ctx.stroke();
    ctx.restore();

    // Attach name last
    ctx.strokeText(name, x - width / 2, y + 40);
}

export function render_bullet(ctx, x, y, color) {
    // Make the circle first
    ctx.beginPath();
    ctx.arc(x, y, 5, 0, 2 * Math.PI);
    ctx.save()
    ctx.fillStyle = color;
    ctx.fill();
    ctx.stroke();
    ctx.restore();
}

export function render_border(ctx, origin_x, origin_y) {
    // Render boundary
    let width = get_settings().arena_width;
    let height = get_settings().arena_height;
    ctx.strokeRect(origin_x, origin_y, width, height);
}

export function render_background(ctx, x, y) {
    ctx.save();
    ctx.strokeStyle = 'rgba(0,0,0,0.4)';
    ctx.beginPath();
    for (let i = 0; i < 5000; i += 100) {
        ctx.moveTo(i - x % 100, 0);
        ctx.lineTo(i - x % 100, 4000);
    }

    for (let i = 0; i < 5000; i += 100) {
        ctx.moveTo(0, i - y % 100);
        ctx.lineTo(5000, i - y % 100);
    }

    ctx.stroke();
    ctx.restore();
}


const lerp = (a, b, amount) => (1 - amount) * a + amount * b;

export function interpolate(y1, y2, time1, time2, interop_point) {
    let delta_x = time2 - time1;
    let ratio = (interop_point - time1) / delta_x;
    return lerp(y1, y2, ratio);
}

export function translator(my_x, my_y, center_x, center_y) {
    let d_x = center_x - my_x;
    let d_y = center_y - my_y;

    return function (x, y) {
        return { x: x + d_x, y: y + d_y};
    };
}
