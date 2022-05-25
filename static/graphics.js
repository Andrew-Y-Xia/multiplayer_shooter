// ALL FUNCTIONS IN THIS FILE DRAW RELATIVE TO PLAYER



export function render_sprite(ctx, x, y, dir, name, color) {

    ctx.save();
    // Add barrel
    ctx.translate(x, y);
    ctx.rotate(dir);
    ctx.translate(-x, -y);

    const height = 40;
    const width = 20;
    ctx.strokeRect(x - (width / 2), y , width, height);
    ctx.restore();
    
    

    // Make the circle first
    ctx.beginPath();
    ctx.arc(x, y, 20, 0, 2 * Math.PI);
    ctx.fillStyle = 'white';
    ctx.fill();
    ctx.stroke();

    // Attach name last
    ctx.strokeText(name, x, y - 10);
}

function render_bullet(ctx, x, y, color) {
    
}

function render_wall(ctx, x, y) {

}
