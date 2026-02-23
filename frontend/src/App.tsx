import React, { useState, useRef } from 'react';
import type { MouseEvent } from 'react';
import { v4 as uuidv4 } from 'uuid';
import { Hexagon, Zap, ArrowRight, Settings, Play, Server, Clock } from 'lucide-react';
import './index.css';

type NodeType = 'webhook' | 'http' | 'transform' | 'ai' | 'cron';

interface NodeItem {
  id: string;
  type: NodeType;
  title: string;
  x: number;
  y: number;
  config?: any;
}

interface EdgeItem {
  id: string;
  from: string;
  to: string;
}

const SIDEBAR_NODES = [
  { type: 'webhook' as NodeType, title: 'Webhook Trigger', icon: Zap, desc: 'Start workflow on HTTP request' },
  { type: 'cron' as NodeType, title: 'Cron Trigger', icon: Clock, desc: 'Schedule workflows' },
  { type: 'http' as NodeType, title: 'HTTP Request', icon: Server, desc: 'Make an external API call' },
  { type: 'transform' as NodeType, title: 'Data Transform', icon: ArrowRight, desc: 'Map JSON fields' },
  { type: 'ai' as NodeType, title: 'AI Assistant', icon: Hexagon, desc: 'OpenAI compatible chat' },
];

function App() {
  const [nodes, setNodes] = useState<NodeItem[]>([]);
  const [edges, setEdges] = useState<EdgeItem[]>([]);

  const [draggedType, setDraggedType] = useState<NodeType | null>(null);

  const [draggingNodeId, setDraggingNodeId] = useState<string | null>(null);
  const [offset, setOffset] = useState({ x: 0, y: 0 });

  const [connectingFrom, setConnectingFrom] = useState<string | null>(null);
  const [mouseX, setMouseX] = useState(0);
  const [mouseY, setMouseY] = useState(0);

  const canvasRef = useRef<HTMLDivElement>(null);

  // Dragging from Sidebar
  const handleDragStartSidebar = (type: NodeType) => {
    setDraggedType(type);
  };

  const handleDragOverCanvas = (e: React.DragEvent) => {
    e.preventDefault();
  };

  const handleDropCanvas = (e: React.DragEvent) => {
    e.preventDefault();
    if (!draggedType || !canvasRef.current) return;

    const rect = canvasRef.current.getBoundingClientRect();
    const nx = e.clientX - rect.left - 120; // center roughly
    const ny = e.clientY - rect.top - 40;

    const nodeMeta = SIDEBAR_NODES.find(n => n.type === draggedType);
    if (!nodeMeta) return;

    const newNode: NodeItem = {
      id: uuidv4(),
      type: draggedType,
      title: nodeMeta.title,
      x: nx,
      y: ny,
      config: {}
    };

    setNodes([...nodes, newNode]);
    setDraggedType(null);
  };

  // Dragging Nodes around Canvas
  const handleNodeMouseDown = (e: MouseEvent, id: string) => {
    e.stopPropagation();
    const node = nodes.find(n => n.id === id);
    if (!node) return;

    setDraggingNodeId(id);
    setOffset({
      x: e.clientX - node.x,
      y: e.clientY - node.y
    });
  };

  const handleCanvasMouseMove = (e: MouseEvent) => {
    if (connectingFrom && canvasRef.current) {
      const rect = canvasRef.current.getBoundingClientRect();
      setMouseX(e.clientX - rect.left);
      setMouseY(e.clientY - rect.top);
    }

    if (draggingNodeId) {
      setNodes(prev => prev.map(n =>
        n.id === draggingNodeId
          ? { ...n, x: e.clientX - offset.x, y: e.clientY - offset.y }
          : n
      ));
    }
  };

  const handleCanvasMouseUp = () => {
    setDraggingNodeId(null);
    setConnectingFrom(null);
  };

  // Connection Edges
  const handlePortMouseDown = (e: MouseEvent, nodeId: string, isOutput: boolean) => {
    e.stopPropagation();
    if (isOutput) {
      setConnectingFrom(nodeId);
      if (canvasRef.current) {
        const rect = canvasRef.current.getBoundingClientRect();
        setMouseX(e.clientX - rect.left);
        setMouseY(e.clientY - rect.top);
      }
    }
  };

  const handlePortMouseUp = (e: MouseEvent, targetNodeId: string, isInput: boolean) => {
    e.stopPropagation();
    if (connectingFrom && connectingFrom !== targetNodeId && isInput) {
      // Create edge
      setEdges(prev => [...prev, { id: uuidv4(), from: connectingFrom, to: targetNodeId }]);
    }
    setConnectingFrom(null);
  };

  const executeGraph = async () => {
    // Scaffold UI visual feedback for running nodes
    alert("Running Workflow!\nCheck Rust API endpoints mapping when connecting backend.");
  };

  const renderIcon = (type: string) => {
    const meta = SIDEBAR_NODES.find(s => s.type === type);
    if (!meta) return <Settings size={18} />;
    const Icon = meta.icon;
    return <Icon size={18} />;
  };

  return (
    <div className="app-container">
      {/* Header */}
      <header className="header">
        <div className="header-title">
          <div className="header-logo"><Play size={18} fill="#fff" /></div>
          rusty-automation
        </div>
        <div className="header-actions">
          <button className="btn success" onClick={executeGraph}>
            <Play size={16} style={{ marginRight: 6, display: 'inline' }} /> Execute
          </button>
        </div>
      </header>

      {/* Main Workspace */}
      <div className="workspace">
        {/* Sidebar View */}
        <aside className="sidebar">
          <div className="sidebar-header">
            <h2>Add Node</h2>
          </div>
          <div className="node-list">
            {SIDEBAR_NODES.map(node => (
              <div
                key={node.type}
                className="draggable-node"
                draggable
                onDragStart={() => handleDragStartSidebar(node.type)}
              >
                <div className="node-icon">
                  <node.icon size={20} />
                </div>
                <div className="node-info">
                  <span className="node-title">{node.title}</span>
                  <span className="node-desc">{node.desc}</span>
                </div>
              </div>
            ))}
          </div>
        </aside>

        {/* Canvas Area */}
        <main
          className="canvas"
          ref={canvasRef}
          onDragOver={handleDragOverCanvas}
          onDrop={handleDropCanvas}
          onMouseMove={handleCanvasMouseMove}
          onMouseUp={handleCanvasMouseUp}
          onMouseLeave={handleCanvasMouseUp}
        >
          {/* SVG layer for Connections */}
          <svg className="canvas-svg">
            <defs>
              <linearGradient id="edge-gradient" x1="0%" y1="0%" x2="100%" y2="0%">
                <stop offset="0%" stopColor="var(--success)" />
                <stop offset="100%" stopColor="var(--primary)" />
              </linearGradient>
            </defs>
            {/* Active Drawing Line */}
            {connectingFrom && (
              <path
                className="edge-path"
                d={`M ${nodes.find(n => n.id === connectingFrom)?.x! + 240} ${nodes.find(n => n.id === connectingFrom)?.y! + 45} C ${nodes.find(n => n.id === connectingFrom)?.x! + 300} ${nodes.find(n => n.id === connectingFrom)?.y! + 45}, ${mouseX - 50} ${mouseY}, ${mouseX} ${mouseY}`}
              />
            )}
            {/* Established Edges */}
            {edges.map(edge => {
              const fromN = nodes.find(n => n.id === edge.from);
              const toN = nodes.find(n => n.id === edge.to);
              if (!fromN || !toN) return null;
              const startX = fromN.x + 240;
              const startY = fromN.y + 45;
              const endX = toN.x;
              const endY = toN.y + 45;
              return (
                <path
                  key={edge.id}
                  className="edge-path static"
                  d={`M ${startX} ${startY} C ${startX + 60} ${startY}, ${endX - 60} ${endY}, ${endX} ${endY}`}
                />
              )
            })}
          </svg>

          {/* Render Active Nodes */}
          {nodes.map(node => (
            <div
              key={node.id}
              className={`canvas-node ${draggingNodeId === node.id ? 'selected' : ''}`}
              style={{ left: node.x, top: node.y }}
              onMouseDown={(e) => handleNodeMouseDown(e, node.id)}
            >
              {/* Output port */}
              <div
                className="canvas-port output"
                onMouseDown={(e) => handlePortMouseDown(e, node.id, true)}
              />
              {/* Input port */}
              <div
                className="canvas-port input"
                onMouseUp={(e) => handlePortMouseUp(e, node.id, true)}
              />

              <div className="canvas-node-header">
                {renderIcon(node.type)}
                <span className="node-title">{node.title}</span>
              </div>

              <div className="canvas-node-body">
                <span className="node-desc">ID: {node.id.split('-')[0]}...</span>
              </div>
            </div>
          ))}
        </main>
      </div>
    </div>
  );
}

export default App;
