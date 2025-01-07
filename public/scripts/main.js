const ALL_UNIT_DATA_PATH = "/all_units";
let CURRENT_LERP_UNITS = [];

// Retreive all unit names
async function fetch_unit_array() {
    outcome = (await fetch(ALL_UNIT_DATA_PATH)).json();
    return outcome;
}

// Construct the unit div (where a hero can then be selected)
// from an inputted unit array
function construct_unit_div(unit_array, num, default_name) {
    let unit_div = document.createElement("div");
    unit_div.classList.add("unit_div");

    let unit_number = document.createElement("p");
    unit_number.textContent = "Unit " + num;
    
    let unit_select = document.createElement("select")
    unit_select.classList.add("unit_select");
    unit_select.ariaLabel = "Unit Selection:"
    
    // populate our select element
    for (let i = 0; i < unit_array.length; i++) {
        let unit_option = document.createElement("option");
        unit_option.value = unit_array[i];
        unit_option.innerText = unit_array[i];
        unit_select.appendChild(unit_option);
    }
    
    unit_select.value = default_name;
    unit_div.appendChild(unit_number);
    unit_div.appendChild(unit_select);
    return unit_div;
}

function construct_unit_submit() {
    let submit_div = document.createElement("div");
    submit_div.innerHTML = "<button type=\"button\" class=\"feh_operation_submit\"> Submit </button>"
    submit_div.classList.add("operation_submit");
    submit_div.classList.add("input_elem");

    console.log(submit_div.firstChild);
    return submit_div.firstChild;
}

// Removes all children of a given node 
function clear_children(node) {
    while (node.firstChild) {
        node.removeChild(node.lastChild);
    }

    return node;
}

function norm_to_range(val, old_min, old_max, new_min, new_max) {
    return (val - old_min) / (old_max - old_min) * (new_max - new_min) + new_min;
}

async function construct_lerp_result(op_query) {
    const op_url = "/?lerp&startunit=" + op_query[0] + "&endunit=" + op_query[1];
    lerp_units = await (await fetch(op_url)).json();
    CURRENT_LERP_UNITS = lerp_units;
    console.log(lerp_units);
    
    let result_div = document.getElementsByClassName("operation_result")[0];
    let true_lerp_slider = document.createElement("input");
    let lerp_slider = document.createElement("p");

    true_lerp_slider.setAttribute("type", "range");
    true_lerp_slider.setAttribute("value", "0");
    true_lerp_slider.setAttribute("max", "10");
    true_lerp_slider.setAttribute("name", "unit_lerp_slider");
    true_lerp_slider.setAttribute("step", "1");
    true_lerp_slider.id = "unit_lerp_slider";
    
    let max_dist = 5;
    let outcome_unit_list = document.createElement("ul");
    
    for (let i = 0; i < lerp_units.length; i++) {
        max_dist = Math.max(max_dist, lerp_units[i][1]);
    }

    lerp_slider.appendChild(true_lerp_slider);
    
    clear_children(result_div).appendChild(lerp_slider);
    result_div.appendChild(outcome_unit_list);
    console.log("The result was: " + op_query);
    
    let vline_set = document.getElementsByClassName("vdist_line");
    console.log(vline_set)
    let vline_half_width = vline_set[0].getBoundingClientRect().width / 2;
    let circle_set = document.getElementsByClassName("circle_set")[0];
    let circle_children = circle_set.children;
    let circle_radius = circle_set.firstElementChild.getBoundingClientRect().width / 2;
    let pbr = circle_set.parentElement.getBoundingClientRect()
    let llr = circle_set.parentElement.lastElementChild.getBoundingClientRect();
    
    for (let i = 0; i < circle_children.length; i++) {
        let PARTITION_WIDTH = i / 10 * pbr.width;
        circle_children[i].style.left = ((PARTITION_WIDTH - circle_radius) / pbr.width * 100).toString() + "%";
        circle_children[i].style.bottom = llr.height / 2 - circle_radius + norm_to_range(lerp_units[i][1], 0, max_dist, 0, pbr.height);
        
        vline_set[i].style.left = ((PARTITION_WIDTH - vline_half_width) / pbr.width * 100).toString() + "%";
        vline_set[i].style.height = circle_children[i].style.bottom;

        console.log(circle_children[i]);
        console.log(vline_set[i]);
    }

    true_lerp_slider.addEventListener("input", function() {
        let index_at = Number(true_lerp_slider.value);

        document.getElementById("cur_unit_partition").textContent = (index_at / 10 * 100).toFixed(2).toString() + "%";
        document.getElementById("cur_unit").textContent = CURRENT_LERP_UNITS[index_at][0];
        document.getElementById("cur_unit_distance").textContent = CURRENT_LERP_UNITS[index_at][1].toString();

        for (let i = 0; i < circle_children.length; i++) {
            circle_children[i].style.backgroundColor = "#FF0000";
        }

        circle_children[index_at].style.backgroundColor = "#000000";
    });

    for (let i = 0; i < circle_children.length; i++) {
        circle_children[i].style.backgroundColor = "#FF0000";
    }

    document.getElementById("cur_unit").textContent = CURRENT_LERP_UNITS[0][0];
    circle_children[0].style.backgroundColor = "#000000";

    document.getElementById("lerp_output_content").style.visibility = "visible";
}

class FehOperation {
    num_units;
    op_name;
    output_construction_func;

    constructor(num_units, op_name, output_func) {
        this.num_units = num_units;
        this.op_name = op_name;
        this.output_construction_func = output_func;
    }
}

// where shit goes down
async function main() {
    let units = await fetch_unit_array();
    let ops = [new FehOperation(2, "lerp", construct_lerp_result)];
    let unit_input_field = document.getElementById("unit_input_fields");

    let SELECT_DEFAULTS = ["Tiki (A)", "Hector"];

    for (let op_idx = 0; op_idx < ops.length; op_idx++) {
        let current_op = ops[op_idx];
        let op_div = document.createElement("div");
        op_div.classList.add(current_op.op_name + "_div");
        op_div.classList.add("input_elem");

        for (let unit_n = 1; unit_n <= current_op.num_units; unit_n++) {
            unit_div = construct_unit_div(units, unit_n, SELECT_DEFAULTS[unit_n - 1]);
            op_div.appendChild(unit_div);
        }
        
        unit_submit = construct_unit_submit();
        unit_submit.addEventListener("click", function (event) {
            let op_div = document.getElementsByClassName(current_op.op_name + "_div")[0];
            let unit_selects = op_div.getElementsByClassName("unit_select");
            let query_units = new Array(2);

            for (let select_idx = 0; select_idx < unit_selects.length; select_idx++) {
                query_units[select_idx] = unit_selects[select_idx].value;
            }

            current_op.output_construction_func(query_units);
        });

        //console.log(unit_submit.parentNode);
        unit_input_field.appendChild(op_div);
        unit_input_field.appendChild(unit_submit.parentNode);
    }

}

main();
