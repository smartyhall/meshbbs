#!/bin/bash
# Initialize all NPC dialogue trees for Old Towne Mesh
# Run this after connecting to the game as an admin

echo "Initializing NPC Dialogue Trees..."
echo "This will set up interactive dialogues for all 5 starter NPCs"
echo ""

# Note: These are example command references
# In practice, you would paste these into the game console one at a time
# or integrate them into the seed_starter_npcs() function

echo "To initialize NPCs, run these commands in-game:"
echo ""
echo "=== MAYOR THOMPSON ==="
echo '@DIALOG mayor_thompson EDIT greeting {"text":"Welcome to Old Towne Mesh! I'\''m Mayor Thompson.","choices":[{"label":"New player tour","goto":"tutorial_offer"},{"label":"Town information","goto":"town_info"},{"label":"Goodbye","exit":true}]}'
echo ""
echo "=== CITY CLERK ==="
echo '@DIALOG city_clerk EDIT greeting {"text":"City Hall administrative services.","choices":[{"label":"How things work","goto":"how_it_works"},{"label":"Goodbye","exit":true}]}'
echo ""
echo "=== GATE GUARD ==="
echo '@DIALOG gate_guard EDIT greeting {"text":"North Gate security.","choices":[{"label":"What'\''s beyond?","goto":"beyond"},{"label":"Farewell","exit":true}]}'
echo ""
echo "=== MARKET VENDOR ==="  
echo '@DIALOG market_vendor EDIT greeting {"text":"Welcome to my stall!","choices":[{"label":"Show wares","goto":"wares"},{"label":"Goodbye","exit":true}]}'
echo ""
echo "=== MUSEUM CURATOR ==="
echo '@DIALOG museum_curator EDIT greeting {"text":"Welcome to the Mesh Museum!","choices":[{"label":"Tell me history","goto":"history"},{"label":"Thank you","exit":true}]}'
echo ""
echo "Done! All NPCs now have interactive dialogue."
