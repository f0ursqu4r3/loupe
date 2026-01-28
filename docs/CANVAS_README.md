# Canvas (Semantic) View

A semantic canvas for exploratory data analysis where nodes represent independent queries connected by narrative relationships.

## Key Concepts

### Semantic Edges (Not Execution Dependencies)

Unlike traditional BI tools where edges represent data flow, the Canvas uses **semantic relationships**:

| Relationship | Meaning |
|-------------|---------|
| `motivates` | This query inspired or led to another query |
| `explains` | This query provides context/explanation for another |
| `contradicts` | Results conflict with or challenge another query |
| `supports` | Results reinforce or validate another query |
| `derives_from` | Query was adapted from another |
| `questions` | Query raises questions about another |

**Important**: Edges are purely narrative. They do NOT create execution dependencies. Each query runs independently.

### Global Time Window

The timeline bar at the top controls a global time range passed to all queries as `start` and `end` parameters. Queries should use `:start` and `:end` placeholders:

```sql
SELECT provider, COUNT(*) AS outages
FROM outage
WHERE ts >= :start AND ts <= :end
GROUP BY provider
```

### Per-Node Execution

- Nodes do NOT run automatically when connected
- Click "Run" in the inspector to execute the selected query
- Each node maintains its own result state
- Results persist in localStorage

## Usage

1. **Create a Query Node**: Click "New Query Node" in the toolbar
2. **Select a Datasource**: Use the dropdown in the inspector
3. **Write SQL**: Use the Monaco editor in the split pane
4. **Run**: Click "Run" to execute against the selected datasource
5. **Connect**: Click the chain icon on a node, then click another node to create a semantic edge
6. **Label Edges**: Click on an edge label to change the relationship type

## Persistence

Canvas state is stored in `localStorage` under the key `loupe-canvas-storage`:
- Nodes: positions, titles, SQL, results
- Edges: connections and relationship labels
- Time range: selected preset and live mode

Each canvas node that has been run creates a corresponding Query in the backend (tagged with `canvas`).

## Files

| File | Purpose |
|------|---------|
| `views/CanvasView.vue` | Main canvas component |
| `stores/canvas.ts` | Pinia store for state management |
| `types/canvas.ts` | TypeScript type definitions |

## Future Enhancements

- [ ] Chart visualizations (line, bar, stat) - currently placeholders
- [ ] Canvas pan/zoom
- [ ] Backend persistence for canvases (currently localStorage only)
- [ ] Export canvas as shareable artifact
- [ ] Node resizing
- [ ] Undo/redo
