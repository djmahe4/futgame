def print_scorecard(team1, team2, score1, score2, goal_scorers, onehasball):
    # Specify the width for each section
    width = 50

    # Calculate the space needed for team names and scores
    team_score_width = width - len(team1) - len(team2) - len(str(score1)) - len(str(score2))

    # Print the top section
    print('=' * width)
    print(
        f"{team1:<{team_score_width // 2}}  {score1:^{len(str(score1))}} - {score2:<{len(str(score2))}} {team2:^{team_score_width // 2}}")
    print('=' * width)

    # Print goal scorers under respective team names
    #print(f"{team1} Scorers:")
    for scorer in goal_scorers:
        if scorer['team'] == team1:
            print(f"{scorer['player']} - {scorer['time']}'\n")

    #print(f"\n{team2} Scorers:")
    for scorer in goal_scorers:
        if scorer['team'] == team2:
            print(f"                {scorer['player']} - {scorer['time']}'\n")
    if onehasball==True:
        print("⚽")

# Example usage
team1_name = "Team A"
team2_name = "Team B"
team1_score = 0
team2_score = 0
goal_scorers = []  # Empty list for the start of the match

#print_scorecard(team1_name, team2_name, team1_score, team2_score, goal_scorers,onehasball=False)
