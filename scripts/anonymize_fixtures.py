#!/usr/bin/env python3
"""
Anonymize Jira API fixtures for safe version control
This script takes raw fixtures and creates anonymized versions suitable for testing
"""

import json
import os
import re
from pathlib import Path
from typing import Any, Dict, List

class JiraFixtureAnonymizer:
    def __init__(self):
        self.anonymized_data = {
            "users": {},
            "projects": {},
            "issues": {},
            "emails": {},
            "urls": {},
            "ids": {},
        }
        
    def anonymize_user(self, user_data: Dict[str, Any]) -> Dict[str, Any]:
        """Anonymize user data"""
        if not isinstance(user_data, dict):
            return user_data
            
        anonymized = user_data.copy()
        
        # Anonymize user identifiers
        if "name" in anonymized:
            original_name = anonymized["name"]
            if original_name not in self.anonymized_data["users"]:
                self.anonymized_data["users"][original_name] = f"user{len(self.anonymized_data['users']) + 1}"
            anonymized["name"] = self.anonymized_data["users"][original_name]
            
        if "key" in anonymized:
            anonymized["key"] = anonymized.get("name", "user1")
            
        if "displayName" in anonymized:
            anonymized["displayName"] = f"Test User {len(self.anonymized_data['users'])}"
            
        if "emailAddress" in anonymized:
            original_email = anonymized["emailAddress"]
            if original_email not in self.anonymized_data["emails"]:
                self.anonymized_data["emails"][original_email] = f"testuser{len(self.anonymized_data['emails']) + 1}@example.com"
            anonymized["emailAddress"] = self.anonymized_data["emails"][original_email]
            
        # Anonymize avatar URLs
        if "avatarUrls" in anonymized and isinstance(anonymized["avatarUrls"], dict):
            for size in anonymized["avatarUrls"]:
                anonymized["avatarUrls"][size] = f"https://jira.example.com/secure/useravatar?ownerId=anon&avatarId=10000"
                
        return anonymized
    
    def anonymize_project(self, project_data: Dict[str, Any]) -> Dict[str, Any]:
        """Anonymize project data"""
        if not isinstance(project_data, dict):
            return project_data
            
        anonymized = project_data.copy()
        
        # Anonymize project identifiers
        if "key" in anonymized:
            original_key = anonymized["key"]
            if original_key not in self.anonymized_data["projects"]:
                self.anonymized_data["projects"][original_key] = f"TEST{len(self.anonymized_data['projects']) + 1}"
            anonymized["key"] = self.anonymized_data["projects"][original_key]
            
        if "name" in anonymized:
            anonymized["name"] = f"Test Project {len(self.anonymized_data['projects'])}"
            
        if "id" in anonymized:
            original_id = anonymized["id"]
            if original_id not in self.anonymized_data["ids"]:
                self.anonymized_data["ids"][original_id] = str(10000 + len(self.anonymized_data["ids"]))
            anonymized["id"] = self.anonymized_data["ids"][original_id]
            
        # Anonymize avatar URLs
        if "avatarUrls" in anonymized and isinstance(anonymized["avatarUrls"], dict):
            for size in anonymized["avatarUrls"]:
                anonymized["avatarUrls"][size] = f"https://jira.example.com/secure/projectavatar?pid=10000&avatarId=10000"
                
        return anonymized
    
    def anonymize_issue(self, issue_data: Dict[str, Any]) -> Dict[str, Any]:
        """Anonymize issue data"""
        if not isinstance(issue_data, dict):
            return issue_data
            
        anonymized = issue_data.copy()
        
        # Anonymize issue key
        if "key" in anonymized:
            original_key = anonymized["key"]
            if original_key not in self.anonymized_data["issues"]:
                project_key = original_key.split("-")[0] if "-" in original_key else "TEST"
                issue_num = len(self.anonymized_data["issues"]) + 1
                self.anonymized_data["issues"][original_key] = f"{project_key}-{issue_num}"
            anonymized["key"] = self.anonymized_data["issues"][original_key]
            
        # Anonymize issue ID
        if "id" in anonymized:
            original_id = anonymized["id"]
            if original_id not in self.anonymized_data["ids"]:
                self.anonymized_data["ids"][original_id] = str(20000 + len(self.anonymized_data["ids"]))
            anonymized["id"] = self.anonymized_data["ids"][original_id]
            
        # Anonymize fields
        if "fields" in anonymized and isinstance(anonymized["fields"], dict):
            fields = anonymized["fields"]
            
            # Anonymize summary and description
            if "summary" in fields:
                fields["summary"] = f"Test Issue Summary {len(self.anonymized_data['issues'])}"
            if "description" in fields:
                fields["description"] = f"Test issue description for {anonymized.get('key', 'TEST-1')}"
                
            # Anonymize project reference
            if "project" in fields and isinstance(fields["project"], dict):
                fields["project"] = self.anonymize_project(fields["project"])
                
            # Anonymize user references
            for user_field in ["assignee", "reporter", "creator"]:
                if user_field in fields and fields[user_field] is not None:
                    fields[user_field] = self.anonymize_user(fields[user_field])
                    
            # Anonymize status, priority, issue type
            for field_name in ["status", "priority", "issuetype"]:
                if field_name in fields and isinstance(fields[field_name], dict):
                    field_data = fields[field_name]
                    if "id" in field_data:
                        original_id = field_data["id"]
                        if original_id not in self.anonymized_data["ids"]:
                            self.anonymized_data["ids"][original_id] = str(30000 + len(self.anonymized_data["ids"]))
                        field_data["id"] = self.anonymized_data["ids"][original_id]
                        
        return anonymized
    
    def anonymize_urls(self, data: Any) -> Any:
        """Anonymize URLs in data"""
        if isinstance(data, str):
            # Replace Adobe Jira URLs with example URLs
            data = re.sub(r'https://jira\.corp\.adobe\.com', 'https://jira.example.com', data)
            data = re.sub(r'https://jira\.adobe\.com', 'https://jira.example.com', data)
            return data
        elif isinstance(data, dict):
            return {k: self.anonymize_urls(v) for k, v in data.items()}
        elif isinstance(data, list):
            return [self.anonymize_urls(item) for item in data]
        else:
            return data
    
    def anonymize_data(self, data: Any) -> Any:
        """Recursively anonymize all data"""
        if isinstance(data, dict):
            # Check if this looks like user data
            if "name" in data and "displayName" in data:
                data = self.anonymize_user(data)
            elif "key" in data and "name" in data and "projectTypeKey" in data:
                data = self.anonymize_project(data)
            elif "key" in data and "fields" in data:
                data = self.anonymize_issue(data)
            else:
                # Recursively anonymize nested data
                data = {k: self.anonymize_data(v) for k, v in data.items()}
        elif isinstance(data, list):
            data = [self.anonymize_data(item) for item in data]
        
        # Anonymize URLs
        data = self.anonymize_urls(data)
        
        return data
    
    def process_file(self, input_path: Path, output_path: Path) -> None:
        """Process a single fixture file"""
        print(f"Processing {input_path} -> {output_path}")
        
        try:
            with open(input_path, 'r', encoding='utf-8') as f:
                data = json.load(f)
            
            # Anonymize the data
            anonymized_data = self.anonymize_data(data)
            
            # Ensure output directory exists
            output_path.parent.mkdir(parents=True, exist_ok=True)
            
            # Write anonymized data
            with open(output_path, 'w', encoding='utf-8') as f:
                json.dump(anonymized_data, f, indent=2, ensure_ascii=False)
                
        except Exception as e:
            print(f"Error processing {input_path}: {e}")
    
    def process_directory(self, input_dir: Path, output_dir: Path) -> None:
        """Process all JSON files in a directory"""
        if not input_dir.exists():
            print(f"Input directory {input_dir} does not exist")
            return
            
        output_dir.mkdir(parents=True, exist_ok=True)
        
        for json_file in input_dir.glob("*.json"):
            if json_file.name.startswith("."):
                continue
                
            output_file = output_dir / json_file.name
            self.process_file(json_file, output_file)
        
        print(f"Processed {len(list(input_dir.glob('*.json')))} files")

def main():
    """Main function"""
    input_dir = Path("fixtures/raw")
    output_dir = Path("fixtures/anonymized")
    
    anonymizer = JiraFixtureAnonymizer()
    anonymizer.process_directory(input_dir, output_dir)
    
    # Save anonymization mapping for reference
    mapping_file = output_dir / "anonymization_mapping.json"
    with open(mapping_file, 'w', encoding='utf-8') as f:
        json.dump(anonymizer.anonymized_data, f, indent=2, ensure_ascii=False)
    
    print(f"Anonymization complete! Output saved to {output_dir}")
    print(f"Anonymization mapping saved to {mapping_file}")

if __name__ == "__main__":
    main()
