strict digraph sense8 {

graph [
charset = "UTF-8";
label = "sense8 object graph",
labelloc = "t",
labeljust = "c",
bgcolor = "#343434",
fontcolor = white,
fontsize = 18,
style = "filled",
rankdir = TB,
margin = 0.2,
splines = spline,
ranksep = 1.0,
nodesep = 0.9
];

node [
colorscheme = "rdylgn11"
style = "solid,filled",
fontsize = 16,
fontcolor = 6,
fontname = "Migu 1M",
color = 7,
fillcolor = 11,
fixedsize = true,
height = 0.6,
width = 1.7
];

edge [
style = solid,
fontsize = 14,
fontcolor = white,
fontname = "Migu 1M",
color = white,
labelfloat = true,
labeldistance = 2.5,
labelangle = 70
];

{% for node in nodes -%}
node[label="{{ node.label }}"]
{{ node.id }} [shape={{ node.shape }}]
{% endfor %}
{% for edge in edges -%}
{{ edge.src }} -> {{ edge.dst }} [{{ edge.src }}={{ edge.dst }}]
{% endfor -%}
}