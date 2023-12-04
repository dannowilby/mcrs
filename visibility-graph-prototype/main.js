
const initialize = () => {
    const canvas = document.getElementById("canvas");
    const ctx = canvas.getContext("2d");

    return ctx;
}

const side_normals = {
    "top": [0, 1],
    "bottom": [0, -1],
    "right": [-1, 0],
    "left": [1, 0],
};

const side_opposite = {
    "top": "bottom",
    "bottom": "top",
    "right": "left",
    "left": "right",
};

const height_threshold = (x, y) => {

    if(y < 100) {
        return 30 / (y + 1);
    }
    
    return 0;
};

const flood_fill = (config, blocks, seed, search) => {

    let output = [];
    let search_queue = [seed];

    while(search_queue.length > 0) {
        const id = search_queue.pop();
        let pos = id.split(",");
        pos = [Number(pos[0]), Number(pos[1])];

        // remove it if it's in our search list
        // if it's not, then we've already seen it
        const index = search.indexOf(id);
        if(index > -1)
            search.splice(index, 1);
        else
            continue;

        // if outside chunk bounds
        if(pos[0] < 0 || pos[0] > config.depth - 1 || 
            pos[1] < 0 || pos[1] > config.depth - 1)
            continue;

        // if the block is solid
        if(blocks[id] == 1)
            continue;

        // add the side if it's not already in our output
        if(pos[0] == 0 && output.indexOf("left") == -1)
            output.push("left");
        if(pos[0] == config.depth - 1 && output.indexOf("right") == -1)
            output.push("right");
        if(pos[1] == 0 && output.indexOf("top") == -1)
            output.push("top");
        if(pos[1] == config.depth - 1 && output.indexOf("bottom") == -1)
            output.push("bottom");

        // push the neighbors
        search_queue.push(`${pos[0] - 1},${pos[1]}`);
        search_queue.push(`${pos[0] + 1},${pos[1]}`);
        search_queue.push(`${pos[0]},${pos[1] - 1}`);
        search_queue.push(`${pos[0]},${pos[1] + 1}`);
    }
    return output;
};

// Create a chunk
const Chunk = (config, x, y) => {

    let depth = config.depth;
    let noise_amplitude = config.noise_amplitude;
    let blocks = {};
    let search = [];

    // populate the chunk
    for(let i = 0; i < depth; i++) {
        for(let j = 0; j < depth; j++) {
            let global_position = [x * depth + i, y * depth + j];

            if (noise.simplex2(global_position[0] * noise_amplitude[0], global_position[1] * noise_amplitude[1]) + height_threshold(global_position[0], global_position[1]) < 0.5) {
                blocks[`${i},${j}`] = 1;
            } else {
                search.push(`${i},${j}`);
            }
        }
    }

    // Create visibility graph
    let visibility_graph = {
        "top": [],
        "bottom": [],
        "left": [],
        "right": [],
    };

    // start flood fill algorithm
    while(search.length > 0) {
        const seed = search[0];

        // flood fill
        const sides = flood_fill(config, blocks, seed, search);

        for(let sides1 = 0; sides1 < sides.length; sides1++) {
            for(let sides2 = sides1 + 1; sides2 < sides.length; sides2++) {
                let s1 = sides[sides1];
                let s2 = sides[sides2];

                if(visibility_graph[s1].indexOf(s2) == -1)
                    visibility_graph[s1].push(s2);
                if(visibility_graph[s2].indexOf(s1) == -1)
                    visibility_graph[s2].push(s1);
            }
        }

    }

    return {
        data: blocks,
        visibility_graph,
        background_color: "grey",
        block_color: "black",
        node: -1,
    };
};

const render_chunk = (ctx, config, chunk, x, y) => {

    let player_chunk = [ Math.floor(player.position[0] / (config.depth * config.block_size)), Math.floor(player.position[1] / (config.depth * config.block_size)) ]

    if(x == player_chunk[0] && y == player_chunk[1]) {
        chunk.background_color = "red";
        chunk.block_color = "darkred";
    }

    let depth = config.depth;
    let block_size = config.block_size;

    ctx.fillStyle = chunk.background_color;
    ctx.fillRect((x * depth * block_size), (y * depth * block_size), (depth * block_size), (depth * block_size));
    ctx.fillStyle = chunk.block_color;

    ctx.strokeRect((x * depth * block_size), (y * depth * block_size), (depth * block_size), (depth * block_size));

    for (const block in chunk.data) {
        const local_position = block.split(',');
        const draw_position = [(x * depth * block_size) + local_position[0] * block_size, (y * depth * block_size) + local_position[1] * block_size];

        ctx.fillRect(draw_position[0], draw_position[1], block_size, block_size);
    }
    ctx.fillStyle = "white";

    ctx.fillText(chunk.node, (x * depth * block_size) + (depth * block_size / 2), (y * depth * block_size) + (depth * block_size / 2));

    ctx.fillStyle = "black";
};

const render_chunks = (ctx, chunks) => {
    for (const chunk in chunks) {
        let pos = chunk.split(",");
        render_chunk(ctx, config, chunks[chunk], pos[0], pos[1]);
    }
}

const render_player = (ctx, config, player) => {

    const p1 = [player.position[0], player.position[1]];
    const p2 = [player.position[0] + (player.view[0] * 10), player.position[1] + (player.view[1] * 10)];

    ctx.strokeStyle = "lightgreen";
    ctx.lineWidth = 2;
    ctx.beginPath();
    ctx.moveTo(p1[0], p1[1]);
    ctx.lineTo(p2[0], p2[1]);
    ctx.stroke();
    ctx.lineWidth = 1;
    ctx.strokeStyle = "black";
}

noise.seed(36);

const config = {
    depth: 20, // size of the chunk might play into how effective the visibility graph is
    block_size: 5,
    noise_amplitude: [0.035, 0.07],
};
const chunk_draw_size = config.depth * config.block_size;
const dimensions = [800, 800];

const ctx = initialize();

const chunks = {};

const to_rad = (x) => (x * (Math.PI / 180));

const look_angle = to_rad(0);
let player = {
    view: [Math.cos(look_angle), Math.sin(look_angle)],
    position: [53, 253],
};

const load_chunks = () => {
    for (let x = 0; x < (dimensions[0] / chunk_draw_size); x++) {
        for (let y = 0; y < (dimensions[1] / chunk_draw_size); y++) {
    
            let id = `${x},${y}`;
            chunks[id] = Chunk(config, x, y);
            
        }
    }
};

const dot = (v1, v2) => v1[0] * v2[0] + v1[1] * v2[1];

const get_neighbors = (pos) => [
    ["top", [pos[0], pos[1] - 1]],
    ["bottom", [pos[0], pos[1] + 1]],
    ["left", [pos[0] - 1, pos[1]]],
    ["right", [pos[0] + 1, pos[1]]],
];

const reset = () => {
    for(const i in chunks) {
        chunks[i].node = -1;
        chunks[i].background_color = "grey";
        chunks[i].block_color = "black";
    }
}

// Implements Tommo's algorithm for chunk culling
// updates the searched chunks with their search number
// we probably want to add some constraints to the direction
// we don't want to go up forward and then up down
const cull = () => {

    const facing = player.view;

    const start_chunk = [ Math.floor(player.position[0] / (config.depth * config.block_size)), Math.floor(player.position[1] / (config.depth * config.block_size)) ];

    let search = [["start", start_chunk, []]];
    let visit_order = [];
    let visited = [];

    let iterations = 0;

    while(search.length > 0) {
        const [side, chunk, constraints] = search.shift();
        const chunk_id = `${chunk[0]},${chunk[1]}`;

        iterations++;

        if(visit_order.indexOf(chunk_id) == -1)
            visit_order.push(chunk_id);

        let neighbors = get_neighbors(chunk);
        for(let i = 0; i < neighbors.length; i++) {
            const [next_side, next_chunk] = neighbors[i];
            const next_chunk_id = `${next_chunk[0]},${next_chunk[1]}`;

            let passable = false;
            for(let i = 0; i < constraints.length; i++) {
                if(constraints[i] == side_opposite[next_side])
                    passable = true;
            }
            if(passable)
                continue;

            // filter the chunk we just came from
            // if(side_opposite[side] == next_side) {
               // continue;
            // }

            // filter the ones we've already seen
            if(visited.indexOf([next_side, next_chunk_id]) != -1){
                continue;
            }
            visited.push([next_side, next_chunk_id]);

            // filter the ones not loaded
            if(next_chunk[0] < 0 || next_chunk[1] < 0 || 
                next_chunk[0] > (dimensions[0] / (config.depth * config.block_size)) - 1 || next_chunk[1] > (dimensions[1] / (config.depth * config.block_size)) - 1) {
                continue;
            }

            // filter forward facing
            // might want to try transforming the normals
            if(dot(facing, side_normals[next_side]) > 0.2){
                continue;
            }

            // filter visibility
            if(side != "start" && chunks[chunk_id].visibility_graph[side].indexOf(next_side) == -1) {
                /*
                console.log("----------------------------");
                console.log(next_chunk_id);
                console.log(next_side);
                console.log(chunk_id);
                console.log(side);
                console.log(chunks[chunk_id].visibility_graph)
                */
                continue;
            }
            
            let next_constraints = [...constraints];
            if(next_constraints.indexOf(next_side) == -1)
                next_constraints.push(next_side);
            search.push([side_opposite[next_side], next_chunk, next_constraints]);
        }

  
    }

    console.log(`iterations: ${iterations}`);
    for (let i = 0; i < visit_order.length; i++) {
        chunks[visit_order[i]].node = i;
    }
    console.log(`Hidden:${visit_order.length / 64}`);
};

// try transforming normals
// debug visibility graphs and why they don't seem to be working right

const update = () => {
    
    let x = Number(document.getElementById("x").value);
    let y = Number(document.getElementById("y").value);
    let look = to_rad(document.getElementById("look").value);

    player = {
        view: [Math.cos(look), Math.sin(look)],
        position: [x, y],
    };

    reset();
    load();
}

const load = () => {

    cull();
    
    render_chunks(ctx, chunks);
    render_player(ctx, config, player);
};

load_chunks();
load();