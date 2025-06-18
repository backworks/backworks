#!/usr/bin/env python3
"""
AI-powered user recommendations handler
Demonstrates machine learning integration with Backworks
"""

import json
import sys
import os
import sqlite3
import random
from datetime import datetime
from typing import List, Dict, Any

def load_user_data(user_id: int = None) -> List[Dict[str, Any]]:
    """Load user data from the database"""
    conn = sqlite3.connect('users.db')
    conn.row_factory = sqlite3.Row
    
    if user_id:
        # Load specific user and their preferences
        query = """
        SELECT u.*, 
               GROUP_CONCAT(up.category || ':' || up.preference_score) as preferences
        FROM users u 
        LEFT JOIN user_preferences up ON u.id = up.user_id 
        WHERE u.id = ?
        GROUP BY u.id
        """
        cursor = conn.execute(query, (user_id,))
        user = cursor.fetchone()
        if user:
            return [dict(user)]
        return []
    else:
        # Load all users with their interaction counts
        query = """
        SELECT u.*, 
               COUNT(ui.id) as interaction_count,
               GROUP_CONCAT(DISTINCT up.category) as categories
        FROM users u 
        LEFT JOIN user_interactions ui ON u.id = ui.user_id 
        LEFT JOIN user_preferences up ON u.id = up.user_id
        GROUP BY u.id
        ORDER BY interaction_count DESC
        """
        cursor = conn.execute(query)
        return [dict(row) for row in cursor.fetchall()]

def calculate_user_similarity(user1: Dict, user2: Dict) -> float:
    """Calculate similarity between two users based on preferences"""
    if not user1.get('preferences') or not user2.get('preferences'):
        return 0.1  # Low similarity if no preferences
    
    prefs1 = {}
    prefs2 = {}
    
    # Parse preferences
    for pref in user1['preferences'].split(','):
        if ':' in pref:
            cat, score = pref.split(':', 1)
            prefs1[cat] = float(score)
    
    for pref in user2['preferences'].split(','):
        if ':' in pref:
            cat, score = pref.split(':', 1)
            prefs2[cat] = float(score)
    
    # Calculate cosine similarity
    common_categories = set(prefs1.keys()) & set(prefs2.keys())
    if not common_categories:
        return 0.1
    
    dot_product = sum(prefs1[cat] * prefs2[cat] for cat in common_categories)
    magnitude1 = sum(prefs1[cat]**2 for cat in prefs1) ** 0.5
    magnitude2 = sum(prefs2[cat]**2 for cat in prefs2) ** 0.5
    
    if magnitude1 == 0 or magnitude2 == 0:
        return 0.1
    
    return dot_product / (magnitude1 * magnitude2)

def generate_collaborative_recommendations(user_id: int) -> List[Dict[str, Any]]:
    """Generate recommendations based on collaborative filtering"""
    users = load_user_data()
    target_user = next((u for u in users if u['id'] == user_id), None)
    
    if not target_user:
        return []
    
    # Find similar users
    similarities = []
    for user in users:
        if user['id'] != user_id:
            similarity = calculate_user_similarity(target_user, user)
            similarities.append((user, similarity))
    
    # Sort by similarity and take top 3
    similarities.sort(key=lambda x: x[1], reverse=True)
    similar_users = similarities[:3]
    
    recommendations = []
    for similar_user, similarity in similar_users:
        recommendations.append({
            "type": "user_connection",
            "user": {
                "id": similar_user['id'],
                "name": similar_user['name'],
                "email": similar_user['email']
            },
            "similarity_score": round(similarity, 3),
            "reason": f"Similar interests and interaction patterns"
        })
    
    return recommendations

def generate_content_recommendations(user_id: int = None) -> List[Dict[str, Any]]:
    """Generate content recommendations based on user preferences"""
    
    # Simulated content database
    content_items = [
        {"id": "tech-001", "type": "article", "title": "Future of AI", "category": "technology", "score": 0.9},
        {"id": "tech-002", "type": "article", "title": "Web3 Explained", "category": "technology", "score": 0.8},
        {"id": "sports-001", "type": "article", "title": "Olympics 2024", "category": "sports", "score": 0.85},
        {"id": "music-001", "type": "playlist", "title": "Indie Rock Hits", "category": "music", "score": 0.9},
        {"id": "travel-001", "type": "guide", "title": "Hidden Gems in Europe", "category": "travel", "score": 0.87},
        {"id": "cook-001", "type": "recipe", "title": "Quick Healthy Meals", "category": "cooking", "score": 0.75},
    ]
    
    if user_id:
        # Get user preferences
        user_data = load_user_data(user_id)
        if user_data and user_data[0].get('preferences'):
            user_prefs = {}
            for pref in user_data[0]['preferences'].split(','):
                if ':' in pref:
                    cat, score = pref.split(':', 1)
                    user_prefs[cat] = float(score)
            
            # Score content based on user preferences
            scored_content = []
            for item in content_items:
                preference_score = user_prefs.get(item['category'], 0.1)
                final_score = item['score'] * preference_score
                scored_content.append({
                    **item,
                    "recommendation_score": round(final_score, 3),
                    "reason": f"Matches your {item['category']} preferences"
                })
            
            # Sort by recommendation score and return top 5
            scored_content.sort(key=lambda x: x['recommendation_score'], reverse=True)
            return scored_content[:5]
    
    # Default recommendations for general users
    return random.sample(content_items, min(3, len(content_items)))

def generate_trending_recommendations() -> List[Dict[str, Any]]:
    """Generate trending content recommendations"""
    return [
        {
            "id": "trend-001",
            "type": "trending",
            "title": "Most Popular This Week",
            "items": [
                {"title": "AI Breakthrough in Healthcare", "views": 15420, "category": "technology"},
                {"title": "Sustainable Living Tips", "views": 12350, "category": "lifestyle"},
                {"title": "New Music Releases", "views": 9876, "category": "music"}
            ],
            "reason": "Trending content across all categories"
        }
    ]

def main():
    """Main handler function"""
    try:
        # Read request data from stdin
        input_data = sys.stdin.read()
        if input_data.strip():
            request = json.loads(input_data)
        else:
            request = {}
        
        method = request.get('method', 'GET')
        path = request.get('path', '')
        query = request.get('query', {})
        body = request.get('body', {})
        
        response_data = {
            "status": 200,
            "headers": {
                "Content-Type": "application/json",
                "X-Powered-By": "Backworks-AI-Python"
            }
        }
        
        # Route based on path
        if '/recommendations' in path and method == 'GET':
            # Extract user ID from path if present
            user_id = None
            path_parts = path.split('/')
            if 'users' in path_parts:
                try:
                    user_idx = path_parts.index('users')
                    if user_idx + 1 < len(path_parts) and path_parts[user_idx + 1].isdigit():
                        user_id = int(path_parts[user_idx + 1])
                except (ValueError, IndexError):
                    pass
            
            # Generate different types of recommendations
            recommendations = {
                "user_id": user_id,
                "timestamp": datetime.now().isoformat(),
                "recommendations": {
                    "collaborative": generate_collaborative_recommendations(user_id) if user_id else [],
                    "content": generate_content_recommendations(user_id),
                    "trending": generate_trending_recommendations()
                },
                "metadata": {
                    "algorithm": "collaborative_filtering_v1",
                    "confidence": 0.85,
                    "processing_time_ms": random.randint(50, 200)
                }
            }
            
            response_data["body"] = recommendations
            
        else:
            # Default response
            response_data["body"] = {
                "error": "Endpoint not found",
                "available_endpoints": [
                    "GET /users/recommendations",
                    "GET /users/{id}/recommendations"
                ]
            }
            response_data["status"] = 404
        
        # Output response as JSON
        print(json.dumps(response_data))
        
    except Exception as e:
        # Error response
        error_response = {
            "status": 500,
            "headers": {"Content-Type": "application/json"},
            "body": {
                "error": "Internal server error",
                "message": str(e),
                "handler": "python_recommendations"
            }
        }
        print(json.dumps(error_response))

if __name__ == "__main__":
    main()
